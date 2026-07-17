## Context

server 端已就位：/auth/device（initiate）、/auth/device/token（poll）、/auth/refresh（rotation）、/auth/revoke，web 端 /activate 核准頁；protocol 有 device DTOs（DeviceAuthorizationResponse、DeviceTokenRequest/Status/Response）。client 端空白：speclink-remote 只有 PAT 檔（credentials.yaml，CLI 專用）與帶三 headers 的 Client；binding 的 Capabilities.authentication 目前為空 vec 且 /binding 本身需要認證——登入前無法靠它探測。Desktop 端 workspace-session 刀已落地（session 模型、設定頁 session 綁定），無任何連線概念。keyring 依賴不存在於 workspace。

實作時發現的缺口：/whoami 只存在於 project scope（Binding extractor 需 project key、membership、API version、repo 解析），web 層（/account）只吃 session cookie——server 沒有任何「bearer → 身分」的 root 層端點。desktop 連線是 origin 層級、登入當下無 project，原「以 access token 打 /whoami」不可實作，故本刀加回一個唯讀 server 端點（決策 8），撤銷 proposal 原「server 端零改動」假設。

## Goals / Non-Goals

**Goals:**

- device flow client 落 speclink-remote（CLI 日後共用）；Desktop 完成 registry＋Keychain＋device login＋PAT fallback＋登出全鏈。
- roadmap Phase 3 gate：credential 唯一落點 OS Keychain；PAT 不進 localStorage、repo、URL、log。
- 手動驗證以 remote-dev-harness 對本地 server 走真實 device login。

**Non-Goals:**

- 不動 server（Capabilities.authentication 空 vec 照舊——登入前探測不依賴它，見決策 3）；不動 CLI 的 auth login 與 credentials.yaml；無 RemoteDataSource／binding handshake 消費（下一刀）；無 Workspace chooser 與 §10.6 設定資訊架構重整（後續刀）；不做多帳號同 server（一 origin 一 credential）。

## Decisions

### 決策 1：device flow client 落 speclink-remote

initiate／poll／refresh／revoke 四函式以 protocol device DTOs 實作於 speclink-remote 新模組，poll 尊重 server 回的 interval、對 pending／granted／denied／expired 逐狀態 typed 回傳。理由：speclink-remote 是 typed client 唯一家（架構 §13.3「CLI 可共用 device login」）；替代案「實作在 src-tauri」被否——CLI 之後要重寫一次。

### 決策 2：credential 唯一落點 OS Keychain，Rust 側進出

src-tauri 新增 CredentialStore trait（get／set／delete，鍵＝server origin＋credential 種類：refresh 或 pat），生產實作用 keyring crate（service 名固定 speclink-desktop，macOS Keychain／Windows Credential Manager），測試實作 in-memory——CI 無 headless Keychain 可用，trait 注入是唯一可測形狀。access token（spk_at_）短效僅存 Rust 記憶體、不落任何盤；refresh rotation 成功後新 refresh credential 立即回寫 Keychain（server 端 family revocation 使舊 rt 重用即撤家族——回寫失敗屬 corrupt 邊界，令使用者重登入）。secret 不進 TS：TS 狀態與 adapter 介面只有連線狀態與身分顯示名；PAT 僅於貼上時單次過境 invoke 參數，不回讀、不入 log。

### 決策 3：登入前探測＝直接 POST /auth/device，不靠 binding capability

/binding 需認證、Capabilities.authentication 又是空 vec，登入前唯一誠實的探測是嘗試 initiate：2xx 即支援 device flow（開瀏覽器）、404／405 即不支援（顯示 PAT 貼上）。網路不可達與 5xx 顯示連線錯誤、不進 fallback——fallback 只給「明確不支援」。替代案「先填 server 的 authentication 宣告」被否：本刀純消費端，且登入前拿不到該宣告，填了也解決不了探測問題。

### 決策 4：connection registry 檔案形狀與位置

registry 檔存 Tauri app 設定目錄（appConfigDir）下的 connections.json：條目＝{id、baseUrl（正規化為 origin）、name、lastActorDisplay?}；一 origin 一條目（重複新增即更新顯示名）；檔案不含任何 token 欄位——測試以序列化往返斷言欄位全集。壞 JSON 歸零清單（與分頁持久化同一寬容哲學）；移除條目連帶登出（決策 6）。替代案「與 CLI 共用 credentials.yaml」被否：那是 secret 檔且屬 CLI；registry 是無 secret 的 profile 清單，混放違反 §10.4 分工。

### 決策 5：device login 編排與瀏覽器開啟

device_login 命令流程：Keychain 已有 refresh credential 時先試靜默 rotation（成功即免瀏覽器——規格「rotation 後…無需重新登入」的落點）；rotation 失敗須分辨語意（決策 3 的原則同樣適用）：明確 permission_denied 才是「credential 已死」、清掉殘骸走完整流程，5xx／不可達／Keychain 故障一律保留 credential 並回報連線錯誤。完整流程：initiate → 以系統瀏覽器開 verification 頁（tauri-plugin-opener；URL 含 user_code 預填參數則直接帶上）→ 依 interval 輪詢 poll → granted 即存 refresh credential 入 Keychain、以 access token 打 root 層 /auth/whoami（決策 8）取身分顯示名寫回 registry → 回報 TS「已登入＋顯示名」。denied／expired 逐一回報可讀錯誤；輪詢中使用者取消即停止。瀏覽器開啟以注入函式抽象——測試注入假開啟器，整條編排（含 rotation）以 in-process speclink-server（memory identity）整合測試驗證，核准步驟由測試直接對 identity store 核准模擬 /activate。

### 決策 6：登出與移除語意

登出＝盡力撤銷（refresh credential 走 /auth/revoke 撤 device family；PAT 無自助撤銷端點——Phase 2 的 PAT revoke 在 /account web 頁，API 面沒有——故 PAT 登出僅刪 Keychain entry 並提示於 /account 頁撤銷）＋必刪 Keychain entry＋清 registry 的身分顯示名；server 不可達時撤銷失敗不阻擋本機刪除（盡力語意）。移除連線＝先走登出、再刪 registry 條目。

### 決策 7：UI 最小面——設定頁「伺服器」頁籤

SettingsView 新增 servers 頁籤（app 全域，不經 session 綁定的設定面）：清單（名稱、origin、登入狀態與身分）、新增（URL＋顯示名→隨即嘗試登入）、登入按鈕（device 預設、探測不支援時就地顯示 PAT 輸入）、登出、移除。表單控制項一律用 packages/web 以外本專案既有的 packages/ui 自建元件（Input 等）；文案走既有 i18n（APP_MESSAGES 的 zh-TW／en），與其他桌面元件同一模式——規格要求的繁中呈現由 zh-TW 字典滿足。§10.6 的三頁籤資訊架構屬後續刀，本頁籤屆時併入 Application scope。

### 決策 8：root 層身分查詢 GET /auth/whoami（server 端唯一新增）

server 新增 root 層 GET /auth/whoami：以 Authorization bearer 解析身分，回 `{"user":{"name":"<顯示名>","handle":"<user id>"}}`（protocol 新 DTO AuthWhoamiResponse，camelCase＋JSON Schema）。bearer 解析與 Binding extractor 第一步完全一致——`spk_at_` 前綴走 access token 驗證、其餘走 PAT 驗證，任何失敗同一 401 permission_denied 不區分原因；PAT 命中同樣前進 last-used。不要求 project scope、API version header 與 repo header——它是登入完成當下、尚未選定 project 的 client 取得身分顯示名的唯一來源，也是 pat_login 的驗證落點。speclink-remote 的 device 模組同步提供 whoami(base_url, bearer) 函式。理由：既有 /whoami 綁 project scope，desktop 連線是 origin 層級。替代案「以假 project key 打 binding、401-vs-404 區分 token 有效性」被否——只能驗證、拿不到顯示名，且依賴 extractor 檢查順序這種未承諾行為；替代案「等下一刀 binding handshake 後補身分」被否——規格 SHALL 要求登入成功即呈現身分顯示名。

## Implementation Contract

- 自動測試：speclink-remote 的 device_flow 整合測試（in-process server：initiate→pending→核准→granted、denied、expired、refresh rotation 舊 rt 重用被拒、whoami 以 access token 查得核准者顯示名）；speclink-server 的 auth_whoami 整合測試（access token → 200 顯示名、PAT → 200 且 last-used 前進、缺席/無效 bearer → 同一 401）；src-tauri 的 CredentialStore in-memory 語意與 registry 往返（含壞 JSON 歸零、序列化欄位全集無 token）；device_login 編排測試（假開啟器＋in-process server）；ServersPanel 的 vitest（假 adapter：新增→device 探測失敗→PAT 輸入現身；登出後狀態歸未登入）。
- secret 衛生驗證：registry 檔序列化測試斷言無 token 欄位；TS adapter 型別無 secret 欄位；grep 確認新增碼無 token 落 log。
- GUI 鐵律（真實視窗、操作前確認使用者未在使用螢幕）：npm run dev 起本地 server → 設定頁新增 http://localhost:8080 → 真實 device login（瀏覽器 /activate 核准）→ 顯示身分；macOS 以 security find-generic-password 確認 Keychain entry 存在；登出後 entry 消失且 server /account 的 device 清單該家族已撤；PAT fallback 路徑實走一次；重啟 app 連線清單保留。
- 回歸：cargo test -p speclink-remote、npm test -w apps/desktop、cargo build --release -p speclink-desktop 全綠。

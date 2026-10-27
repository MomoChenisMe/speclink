# desktop-connections Specification

## Purpose

桌面 app 對 Speclink server 連線的管理：不含 secret 且跨重啟保留的 connection registry、以 device login 為預設並保留 PAT fallback 的登入路徑、登出時的憑證撤銷與連帶清理，以及伺服器管理的最小操作面。本 capability 保證 credential 的唯一落點是 OS Keychain——不落在設定檔或 registry 裡。

## Requirements

### Requirement: connection registry 不含 secret 且跨重啟保留

Desktop SHALL 於 app 設定目錄維護 saved servers 的 connection registry：條目含識別、以 origin 正規化的 server 位址、顯示名與最後登入身分顯示名，SHALL NOT 含任何 token 或 credential 欄位。同一 origin 重複新增 SHALL 更新顯示名而非新增條目。registry SHALL 跨重啟保留；壞 JSON SHALL 歸零清單、不崩潰。

#### Scenario: registry 序列化不含 secret

- **WHEN** 新增連線並完成登入後檢視 registry 檔內容
- **THEN** 檔案僅含識別、origin、顯示名與身分顯示名欄位，無任何 token/credential 欄位

#### Scenario: 重啟保留連線清單

- **WHEN** 新增兩個 server 連線後重啟 app
- **THEN** 伺服器清單呈現同樣兩個條目與其登入狀態


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

---
### Requirement: credential 唯一落點為 OS Keychain

登入取得的 credential（device 流程的 refresh credential、PAT 流程的 PAT）SHALL 逐 server origin 存入 OS Keychain（macOS Keychain／Windows Credential Manager），SHALL NOT 出現於 localStorage、registry 檔、repo、URL、log 或任何 TS 可見狀態；TS 層 SHALL 只見連線狀態與身分顯示名，PAT 僅於使用者貼上時單次過境命令參數。access token SHALL 短效僅存 Rust 記憶體；refresh rotation 成功後 SHALL 立即以新 refresh credential 覆寫 Keychain entry。

#### Scenario: Keychain entry 隨登入建立

- **WHEN** 對本地 dev server 完成 device login
- **THEN** 系統 Keychain 出現該 origin 的 refresh credential entry，且 registry 檔與 localStorage 皆無 token 內容

#### Scenario: rotation 後舊 credential 失效仍可用

- **WHEN** access token 過期後 app 以 refresh credential 換新（rotation）再重啟 app
- **THEN** 重啟後以 Keychain 中最新 refresh credential 成功取得 access token，無需重新登入


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

---
### Requirement: device login 預設與 PAT fallback

新增或登入連線時 Desktop SHALL 先嘗試 device flow（對 server 的 device 初始化端點探測）：支援時 SHALL 以 server 回傳的 verification URI 與 `user_code` 查詢參數開啟系統瀏覽器，並依 server 指示的間隔輪詢至核准、拒絕或逾時，逐一以可讀狀態回報。等待授權期間，發起登入的介面（伺服器頁的連線列，或工作區選擇器的新增並登入區）SHALL 就地呈現等待授權面：顯示裝置碼與驗證網址（兩者各附複製操作）、以有效期限起算的剩餘時間倒數、以及取消操作——使用者無需依賴已開啟的瀏覽器分頁即可換另一部裝置完成核准，也無需切換到其他頁面才看得到登入狀態。取消 SHALL 立即停止輪詢、該連線回到未登入狀態且 SHALL NOT 留存任何 credential。輪詢至逾時仍未核准時 SHALL 顯示逾時的可讀狀態。瀏覽器尚未登入 server 時，登入成功後 SHALL 返回同一裝置核准流程，裝置碼 SHALL 已預填且使用者 SHALL 經過下一步與明確核准／拒絕確認。明確不支援（404/405）時 SHALL 就地顯示 PAT 貼上輸入作為 fallback；網路不可達或 5xx SHALL 顯示連線錯誤、SHALL NOT 進入 PAT fallback。PAT 登入 SHALL 以身分查詢驗證有效後才存入 Keychain。登入成功後 SHALL 呈現該連線的身分顯示名。

#### Scenario: device login 完整走通

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器尚無 server session
- **THEN** 瀏覽器登入後返回已預填裝置碼的核准頁；使用者明確核准後，app 輪詢至 granted、存 refresh credential 入 Keychain、顯示登入身分

#### Scenario: 等待授權面顯示碼與網址

- **WHEN** device login 進入等待授權（瀏覽器已開啟、授權尚未核准）
- **THEN** 該連線列顯示裝置碼與驗證網址並各附複製操作、剩餘時間倒數與取消；複製裝置碼後剪貼簿內容即為該碼

#### Scenario: 從工作區選擇器發起登入同樣呈現等待授權面

- **WHEN** 於工作區選擇器的 server 步驟新增連線並按下登入，且 server 支援 device flow
- **THEN** 選擇器內就地顯示等待授權面（裝置碼、驗證網址、倒數與取消）；取消即停止輪詢並停留在 server 步驟；server 明確不支援時就地顯示 PAT 輸入

#### Scenario: 取消等待不留 credential

- **WHEN** 等待授權期間使用者按下取消
- **THEN** 輪詢停止、連線列回到未登入狀態、Keychain 無任何新增 credential；瀏覽器端該授權請求自然逾期

#### Scenario: 已登入瀏覽器直接進入預填流程

- **WHEN** 對支援 device flow 的 server 按下登入，且系統瀏覽器已有有效 server session
- **THEN** 瀏覽器直接顯示已預填裝置碼的核准頁，並保留下一步與明確核准／拒絕確認

#### Scenario: 不支援 device flow 才現 PAT 輸入

- **WHEN** 對回應 404 於 device 初始化端點的 server 按下登入
- **THEN** 就地顯示 PAT 輸入；輸入有效 PAT 後登入成功並顯示身分

#### Scenario: 瀏覽器端拒絕授權

- **WHEN** device login 輪詢期間使用者於瀏覽器拒絕該裝置
- **THEN** app 停止輪詢並顯示已拒絕的可讀狀態，不留任何 credential


<!-- @trace
source: desktop-settings-mode
updated: 2026-10-10T23:13:14+08:00
-->

---
### Requirement: 登出撤銷與移除連帶清理

登出 SHALL 刪除該 origin 的 Keychain entry 並清除 registry 的身分顯示名；持 refresh credential 者 SHALL 盡力呼叫 server 的 revoke 端點撤銷 device family——server 不可達時撤銷失敗 SHALL NOT 阻擋本機刪除；PAT 登出 SHALL 刪除本機 entry 並提示於 server 帳號頁撤銷。移除連線 SHALL 先執行登出語意再刪 registry 條目。

#### Scenario: 登出撤銷 device family

- **WHEN** 對已 device login 的連線按下登出
- **THEN** Keychain entry 刪除、server 端該 device family 被撤銷（帳號頁 device 清單不再含該裝置）、清單呈現未登入狀態

#### Scenario: server 離線時登出仍完成本機清理

- **WHEN** server 不可達時按下登出
- **THEN** 本機 Keychain entry 與身分顯示名仍被清除，UI 呈現未登入


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

---
### Requirement: 伺服器管理最小面

應用程式設定 SHALL 提供「伺服器」頁（app 全域範圍、與任何專案無關）：頁標題右側「新增連線」鈕展開頁頂的新增卡（URL 與顯示名、新增與取消；新增成功即收起並進入登入流程）。「連線」卡 SHALL 每個 saved server 一列——左為顯示名與狀態籤（已登入／未登入／登入過期／等待授權／失敗）、等寬 origin 與身分名；右為「登出」（已登入）或「登入」／「重新登入」（未登入／登入過期）與「⋯」選單（開啟專案、移除）；等待授權的細節與金鑰輸入 SHALL 於該列下方就地展開；無連線時顯示空狀態。「預設」卡 SHALL 有一列「新增遠端專案時預設使用」下拉（選項為已登入連線與「不預選」），偏好存 app 本機；新增專案對話框的 Server 步驟 SHALL 預選該連線（該連線不存在或未登入時，或對話框帶入口意圖開啟——指定連線、或資料夾的 remote marker 指向的伺服器——時不預選），使用者仍可改選。「登入方式」卡 SHALL 以兩列說明裝置登入（預設）與存取金鑰（伺服器不支援裝置登入時就地改用），不提供其他動作。登入成功時，該連線列 SHALL 伴隨開啟工作區的行動呼籲且該入口 SHALL 取得鍵盤焦點——引導使用者銜接下一步，SHALL NOT 自動開啟工作區選擇器。表單控制項 SHALL 使用專案自建 UI 元件、文案為繁體中文。此頁為最小管理面。

#### Scenario: 新增後清單即時反映

- **WHEN** 於伺服器頁按「新增連線」、填 URL 與顯示名後送出
- **THEN** 新增卡收起、連線卡立即出現該列並進入登入流程；完成登入後該列顯示身分名與「已登入」籤

#### Scenario: 登入成功聚焦開啟工作區

- **WHEN** 任一連線經 device 或 PAT 登入成功
- **THEN** 該連線列的開啟工作區入口取得鍵盤焦點且視覺顯眼；工作區選擇器未自動開啟

#### Scenario: 預設連線於新增專案預選

- **WHEN** 使用者於「預設」卡選定連線 A 為新增遠端專案時預設使用，之後開新增專案對話框選 Server 來源
- **THEN** Server 步驟預選 A（可改選）；若 A 已登出或被移除，則步驟不預選任何連線

#### Scenario: 帶入口意圖開啟時不預選預設連線

- **WHEN** 預設連線為已登入的 Y，使用者開啟一個 remote marker 指向伺服器 X（尚未登入）的本機資料夾，新增專案對話框因此直接進入 Server 步驟
- **THEN** Y 不被預選、頁尾不出現「下一步」；使用者須自行選擇連線或依 X 新增並登入

#### Scenario: 連線列的動作依狀態

- **WHEN** 伺服器頁有一個已登入與一個登入過期的連線
- **THEN** 前者右側為「登出」與「⋯」，後者右側為「重新登入」與「⋯」且狀態籤為警示色；「⋯」選單含開啟專案與移除


<!-- @trace
source: desktop-settings-mode
updated: 2026-10-10T23:13:14+08:00
-->
# tray-status-menu Specification

## Purpose

TBD - created by archiving change 'system-tray-status'. Update Purpose after archive.

## Requirements

### Requirement: 系統匣圖示與原生選單
桌面 app SHALL 於系統匣（macOS 選單列、Windows 通知區域、Linux AppIndicator）顯示 Speclink 圖示；互動樣式 SHALL 由平台決定——macOS 一律為面板（見「面板樣式（macOS）」需求）、非 macOS 平台一律為原生下拉選單（menu-first），SHALL NOT 提供樣式偏好或切換介面。圖示 SHALL 為單色 Speclink 標記（使用者提供的可辨識剪影）；macOS 上 SHALL 以 template 單色形式渲染以適應深淺色選單列。原生選單 SHALL 依序包含：專案區（已開啟的專案分頁）、生命週期分區（提案中／進行中／已就緒）、討論區、動作區（「開啟 Speclink」與「結束」）。

#### Scenario: 非 macOS 啟動後系統匣出現圖示與完整選單
- **WHEN** 使用者於 Windows 或 Linux 啟動桌面 app 且載入完成
- **THEN** 系統匣出現單色 Speclink 標記圖示，展開選單依序可見專案區、生命週期分區、討論區、動作區（「開啟 Speclink」「結束」）

#### Scenario: macOS 啟動後點擊圖示即面板
- **WHEN** 使用者於 macOS 啟動桌面 app 且載入完成，點擊系統匣圖示
- **THEN** 彈出面板而非原生下拉選單，過程中無需任何樣式設定

#### Scenario: 無專案分頁時選單仍可用
- **WHEN** app 尚未開啟任何專案
- **THEN** 選單不顯示專案區與前導分隔線，其餘區段照常可用


<!-- @trace
source: tray-macos-panel-only
updated: 2026-07-16
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/trayStyle.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayStyle.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: 選單內容與看板同源
選單內容 SHALL 由前端資料層（與看板同一 store）供給，並於資料變動時重建；SHALL NOT 為 tray 另建第二條資料查詢路徑。外部寫者改動 openspec/ 觸發既有自動刷新後，選單 SHALL 反映刷新後的狀態，無需主視窗位於前景。

#### Scenario: 背景寫入後選單反映新進度
- **WHEN** 主視窗不在前景，agent 或 CLI 將某進行中變更的任務自 0/12 推進至 3/12，既有自動刷新管線生效
- **THEN** 下次展開系統匣選單時該變更顯示 3/12，與看板一致


<!-- @trace
source: system-tray-status
updated: 2026-07-13
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/128x128.png
  - apps/desktop/src-tauri/icons/128x128@2x.png
  - apps/desktop/src-tauri/icons/32x32.png
  - apps/desktop/src-tauri/icons/64x64.png
  - apps/desktop/src-tauri/icons/Square107x107Logo.png
  - apps/desktop/src-tauri/icons/Square142x142Logo.png
  - apps/desktop/src-tauri/icons/Square150x150Logo.png
  - apps/desktop/src-tauri/icons/Square284x284Logo.png
  - apps/desktop/src-tauri/icons/Square30x30Logo.png
  - apps/desktop/src-tauri/icons/Square310x310Logo.png
  - apps/desktop/src-tauri/icons/Square44x44Logo.png
  - apps/desktop/src-tauri/icons/Square71x71Logo.png
  - apps/desktop/src-tauri/icons/Square89x89Logo.png
  - apps/desktop/src-tauri/icons/StoreLogo.png
  - apps/desktop/src-tauri/icons/icon.icns
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/icons/icon.png
  - apps/desktop/src-tauri/icons/speclink-tray-18.png
  - apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayIcon.ts
-->

---
### Requirement: 生命週期分區與變更進度
選單主體 SHALL 依生命週期階段分區（提案中、進行中、已就緒）：每個非空階段 SHALL 有一個分區標題，其下 SHALL 列出該階段的所有變更。每張變更列 SHALL 顯示其名稱與任務數「n/m」；有任務（總數大於 0）的變更 SHALL 另顯示一條文字進度條反映完成比例。全無變更時 SHALL 顯示明確空狀態文字。

#### Scenario: 進行中變更顯示進度條與任務數
- **WHEN** 作用中專案有一個進行中變更、完成 3 個共 12 個任務
- **THEN** 「進行中」分區標題下該變更顯示其名稱、文字進度條與「3/12」

#### Scenario: 全無變更顯示空狀態
- **WHEN** 作用中專案沒有任何變更
- **THEN** 選單顯示明確空狀態文字而非空白，其餘區段照常


<!-- @trace
source: system-tray-status
updated: 2026-07-13
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/128x128.png
  - apps/desktop/src-tauri/icons/128x128@2x.png
  - apps/desktop/src-tauri/icons/32x32.png
  - apps/desktop/src-tauri/icons/64x64.png
  - apps/desktop/src-tauri/icons/Square107x107Logo.png
  - apps/desktop/src-tauri/icons/Square142x142Logo.png
  - apps/desktop/src-tauri/icons/Square150x150Logo.png
  - apps/desktop/src-tauri/icons/Square284x284Logo.png
  - apps/desktop/src-tauri/icons/Square30x30Logo.png
  - apps/desktop/src-tauri/icons/Square310x310Logo.png
  - apps/desktop/src-tauri/icons/Square44x44Logo.png
  - apps/desktop/src-tauri/icons/Square71x71Logo.png
  - apps/desktop/src-tauri/icons/Square89x89Logo.png
  - apps/desktop/src-tauri/icons/StoreLogo.png
  - apps/desktop/src-tauri/icons/icon.icns
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/icons/icon.png
  - apps/desktop/src-tauri/icons/speclink-tray-18.png
  - apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayIcon.ts
-->

---
### Requirement: 變更子選單動作
每張變更 SHALL 呈現為子選單，其中 SHALL 依序含「開啟此變更」與「複製名稱」動作。選取「開啟此變更」SHALL 顯示主視窗並取得焦點、且開啟該變更的詳情。選取「複製名稱」SHALL 將該變更的 name（純文字，不含進度條字元與任務數）寫入系統剪貼簿，且主視窗隱藏或無焦點時 SHALL 仍寫入成功；剪貼簿寫入失敗時 SHALL NOT 彈出錯誤視窗、SHALL NOT 中斷選單操作。

#### Scenario: 開啟某變更詳情
- **WHEN** 使用者展開某變更子選單並選取「開啟此變更」
- **THEN** 主視窗顯示於前景並取得焦點，且開啟該變更的詳情

#### Scenario: 複製變更名稱
- **WHEN** 使用者展開某變更子選單並選取「複製名稱」
- **THEN** 系統剪貼簿內容等於該變更的 name，不含進度條字元與任務數

##### Example: 選單標籤含進度條但只複製名稱
- **GIVEN** 變更 phase2-e2e-chain 有 10 項任務完成 0 項，選單列標籤呈現「phase2-e2e-chain  ░░░░░░░░ 0/10」
- **WHEN** 使用者選取該變更子選單的「複製名稱」
- **THEN** 剪貼簿內容為「phase2-e2e-chain」

#### Scenario: 主視窗隱藏時複製仍成功
- **WHEN** 主視窗已隱藏或無焦點，使用者自系統匣選取「複製名稱」
- **THEN** 剪貼簿寫入成功，內容等於該變更的 name


<!-- @trace
source: tray-copy-and-panel-mode
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/panel.html
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/capabilities/macos.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/panel.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/trayStyle.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayStyle.ts
  - apps/desktop/src/views/SettingsView.tsx
  - apps/desktop/vite.config.ts
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
  - package-lock.json
-->

---
### Requirement: 討論列表
選單討論區 SHALL 分兩個分區呈現 active 討論：「討論」分區列討論中（尚未轉出變更）的討論；其後「已轉出」分區列已轉出變更的討論（promoted——至少連結一個變更者），無已轉出討論時 SHALL NOT 顯示該分區。兩分區的每則討論 SHALL 呈現為子選單：父項標籤 SHALL 為該討論的 slug（識別錨點直出）；子選單 SHALL 依序含 topic 描述行（disabled、不可選取）、「開啟此討論」、「複製 slug」。選取「開啟此討論」SHALL 顯示主視窗並取得焦點、且開啟該討論。選取「複製 slug」SHALL 將該討論的 slug 寫入系統剪貼簿，主視窗隱藏或無焦點時 SHALL 仍寫入成功。無討論中討論時「討論」分區 SHALL 顯示「討論 0」。

#### Scenario: 討論以 slug 為題、topic 為描述
- **WHEN** 存在 active 討論（slug 為 board-search-bar、topic 為「看板搜尋列」）
- **THEN** 討論區該項父標籤為「board-search-bar」，展開子選單首行為灰字「看板搜尋列」且不可選取

#### Scenario: 開啟某討論
- **WHEN** 使用者展開某討論子選單並選取「開啟此討論」
- **THEN** 主視窗顯示於前景並取得焦點，且開啟該討論

#### Scenario: 複製討論 slug
- **WHEN** 使用者展開某討論子選單並選取「複製 slug」
- **THEN** 系統剪貼簿內容等於該討論的 slug

#### Scenario: 已轉出討論列於已轉出分區
- **WHEN** 存在討論中的討論與已轉出變更的討論各一
- **THEN** 討論中的討論列於「討論」分區、已轉出的列於「已轉出」分區，兩者子選單結構相同（topic 描述行、開啟此討論、複製 slug）

#### Scenario: 無已轉出討論時不顯示該分區
- **WHEN** 目前沒有任何已轉出變更的討論
- **THEN** 選單不出現「已轉出」分區標題

#### Scenario: 無討論時顯示零
- **WHEN** 目前沒有任何討論中的討論
- **THEN** 「討論」分區顯示「討論 0」


<!-- @trace
source: tray-copy-and-panel-mode
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/panel.html
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/capabilities/macos.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/panel.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/trayStyle.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayStyle.ts
  - apps/desktop/src/views/SettingsView.tsx
  - apps/desktop/vite.config.ts
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
  - package-lock.json
-->

---
### Requirement: 選單專案切換
選單專案區 SHALL 列出全部已開啟的專案分頁，作用中專案以勾選標記。點選非作用中專案 SHALL 使桌面 app 切換至該專案（看板與選單一致更新勾選），且 SHALL NOT 將主視窗帶到前景或奪取焦點。點選的專案目錄已失效時 SHALL 沿用看板既有的分頁錯誤處理，app SHALL NOT 崩潰。

#### Scenario: 點選非作用中專案完成切換且不奪焦
- **WHEN** 使用者於系統匣選單點選一個非作用中的專案
- **THEN** 看板切換至該專案、選單勾選移至該專案，主視窗的前景與焦點狀態不變


<!-- @trace
source: system-tray-status
updated: 2026-07-13
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/128x128.png
  - apps/desktop/src-tauri/icons/128x128@2x.png
  - apps/desktop/src-tauri/icons/32x32.png
  - apps/desktop/src-tauri/icons/64x64.png
  - apps/desktop/src-tauri/icons/Square107x107Logo.png
  - apps/desktop/src-tauri/icons/Square142x142Logo.png
  - apps/desktop/src-tauri/icons/Square150x150Logo.png
  - apps/desktop/src-tauri/icons/Square284x284Logo.png
  - apps/desktop/src-tauri/icons/Square30x30Logo.png
  - apps/desktop/src-tauri/icons/Square310x310Logo.png
  - apps/desktop/src-tauri/icons/Square44x44Logo.png
  - apps/desktop/src-tauri/icons/Square71x71Logo.png
  - apps/desktop/src-tauri/icons/Square89x89Logo.png
  - apps/desktop/src-tauri/icons/StoreLogo.png
  - apps/desktop/src-tauri/icons/icon.icns
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/icons/icon.png
  - apps/desktop/src-tauri/icons/speclink-tray-18.png
  - apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayIcon.ts
-->

---
### Requirement: 開啟視窗與結束動作
選單「開啟 Speclink」SHALL 顯示主視窗並使其取得焦點；「結束」SHALL 結束 app。除「開啟 Speclink」、變更子選單動作與討論項外，選單其他操作 SHALL NOT 改變主視窗的顯示與焦點狀態。

#### Scenario: 自最小化狀態開啟主視窗
- **WHEN** 主視窗處於最小化或被其他視窗遮蔽，使用者點選「開啟 Speclink」
- **THEN** 主視窗顯示於前景並取得焦點

#### Scenario: 結束 app
- **WHEN** 使用者點選選單的「結束」
- **THEN** app 行程結束，系統匣圖示消失


<!-- @trace
source: system-tray-status
updated: 2026-07-13
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/128x128.png
  - apps/desktop/src-tauri/icons/128x128@2x.png
  - apps/desktop/src-tauri/icons/32x32.png
  - apps/desktop/src-tauri/icons/64x64.png
  - apps/desktop/src-tauri/icons/Square107x107Logo.png
  - apps/desktop/src-tauri/icons/Square142x142Logo.png
  - apps/desktop/src-tauri/icons/Square150x150Logo.png
  - apps/desktop/src-tauri/icons/Square284x284Logo.png
  - apps/desktop/src-tauri/icons/Square30x30Logo.png
  - apps/desktop/src-tauri/icons/Square310x310Logo.png
  - apps/desktop/src-tauri/icons/Square44x44Logo.png
  - apps/desktop/src-tauri/icons/Square71x71Logo.png
  - apps/desktop/src-tauri/icons/Square89x89Logo.png
  - apps/desktop/src-tauri/icons/StoreLogo.png
  - apps/desktop/src-tauri/icons/icon.icns
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/icons/icon.png
  - apps/desktop/src-tauri/icons/speclink-tray-18.png
  - apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayIcon.ts
-->

---
### Requirement: macOS 進行中數文字徽章
macOS 上系統匣圖示旁 SHALL 顯示作用中專案的進行中變更數文字徽章，並隨資料變動更新；數量為 0 時 SHALL 不顯示文字。非 macOS 平台 SHALL 無此徽章且 SHALL NOT 因此產生錯誤。

#### Scenario: 徽章隨進行中變更數更新
- **WHEN** macOS 上作用中專案的進行中變更數自 2 變為 0
- **THEN** 圖示旁的「2」文字徽章消失；再有變更進入進行中時徽章重新顯示對應數字

<!-- @trace
source: system-tray-status
updated: 2026-07-13
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/128x128.png
  - apps/desktop/src-tauri/icons/128x128@2x.png
  - apps/desktop/src-tauri/icons/32x32.png
  - apps/desktop/src-tauri/icons/64x64.png
  - apps/desktop/src-tauri/icons/Square107x107Logo.png
  - apps/desktop/src-tauri/icons/Square142x142Logo.png
  - apps/desktop/src-tauri/icons/Square150x150Logo.png
  - apps/desktop/src-tauri/icons/Square284x284Logo.png
  - apps/desktop/src-tauri/icons/Square30x30Logo.png
  - apps/desktop/src-tauri/icons/Square310x310Logo.png
  - apps/desktop/src-tauri/icons/Square44x44Logo.png
  - apps/desktop/src-tauri/icons/Square71x71Logo.png
  - apps/desktop/src-tauri/icons/Square89x89Logo.png
  - apps/desktop/src-tauri/icons/StoreLogo.png
  - apps/desktop/src-tauri/icons/icon.icns
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/icons/icon.png
  - apps/desktop/src-tauri/icons/speclink-tray-18.png
  - apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayIcon.ts
-->

---
### Requirement: 面板樣式（macOS）
於 macOS，點擊系統匣圖示 SHALL NOT 顯示原生下拉選單，而 SHALL 於圖示下方彈出貼齊圖示的面板視窗，再次點擊 SHALL 收合——無需任何偏好設定。面板內容 SHALL 與原生選單同源（同一前端資料層：專案、生命週期分區的變更與進度、討論——討論比照原生選單分「討論」與「已轉出」兩分區），SHALL NOT 為面板另建第二條資料查詢路徑。變更與討論列 SHALL 於列尾常駐複製鈕（複製內容與原生選單的複製動作一致：變更為 name、討論為 slug）；複製鈕點擊後 SHALL 短暫顯示成功回饋（勾號圖示，與看板複製鈕同模式）後自行復原。點擊列本體 SHALL 顯示主視窗並開啟對應詳情。面板高度 SHALL 自適應內容（隨內容增減貼合，達上限高度後面板內部捲動、不得於內容未超限時出現多餘捲動與空白）。面板開啟 SHALL NOT 奪取目前前景 app 的焦點；面板失焦時 SHALL 自動收合。面板視窗建立失敗時 app SHALL 以原生選單樣式運作（選單實作跨平台保留、兼作 macOS 失敗後備）並於設定頁本機設定簽浮出單行錯誤。

#### Scenario: 面板樣式下點擊圖示彈出貼齊面板
- **WHEN** 使用者於 macOS 點擊系統匣圖示
- **THEN** 圖示下方彈出貼齊圖示的面板，呈現專案、變更（含進度）與討論清單，未出現原生下拉選單

#### Scenario: 面板不搶焦點且失焦自動收合
- **WHEN** 使用者於其他 app 位於前景時點擊系統匣圖示開啟面板，隨後點擊面板外任意處
- **THEN** 面板開啟期間原前景 app 保持焦點；點擊面板外後面板自動收合

#### Scenario: 面板內以常駐複製鈕複製
- **WHEN** 使用者點擊面板中某討論列列尾的複製鈕（無需 hover 顯示、常駐可見）
- **THEN** 系統剪貼簿內容等於該討論的 slug，複製鈕短暫轉為勾號回饋後復原，面板保持開啟、未開啟主視窗

#### Scenario: 面板高度自適應內容
- **WHEN** 面板開啟且內容筆數少於一屏
- **THEN** 面板高度貼合內容（下方無大片空白），內容增加超過上限高度後面板內部出現捲動

#### Scenario: 面板建立失敗退回原生選單
- **WHEN** macOS 上面板視窗建立失敗
- **THEN** 系統匣以原生選單樣式運作，設定頁本機設定簽浮出單行錯誤訊息


<!-- @trace
source: tray-macos-panel-only
updated: 2026-07-16
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/trayStyle.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayStyle.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: 分區溢出摺疊
系統匣各分區（生命週期各階段、討論、已轉出）SHALL 直列前 5 筆項目；第 6 筆起 SHALL 收進該分區尾端的「還有 N 個…」節點（N＝溢出筆數）。原生選單的溢出節點 SHALL 為子選單，內含其餘項目且各自保有完整子選單動作（變更：開啟此變更／複製名稱；討論：topic 描述行／開啟此討論／複製 slug）；面板的溢出節點 SHALL 為可展開列——點擊展開其餘項目、再點收合，展開後面板高度自適應與上限捲動行為不變。分區項目數不超過 5 時 SHALL NOT 出現溢出節點。

#### Scenario: 超過五筆收進溢出節點
- **WHEN** 進行中分區有 8 張變更
- **THEN** 分區直列前 5 張，尾端出現「還有 3 個…」；展開後可見其餘 3 張且各自可開啟與複製

##### Example: 門檻邊界
| 分區項目數 | 直列 | 溢出節點 |
| ---------- | ---- | -------- |
| 5 | 5 | 無 |
| 6 | 5 | 還有 1 個… |
| 20 | 5 | 還有 15 個… |

#### Scenario: 未超過五筆不出現溢出節點
- **WHEN** 討論分區只有 2 則討論
- **THEN** 兩則直列，無「還有 N 個…」節點

<!-- @trace
source: tray-copy-and-panel-mode
updated: 2026-07-16
code:
  - .dockerignore
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - Cargo.lock
  - apps/desktop/package.json
  - apps/desktop/panel.html
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/capabilities/macos.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/panel.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/trayStyle.test.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/trayStyle.ts
  - apps/desktop/src/views/SettingsView.tsx
  - apps/desktop/vite.config.ts
  - crates/speclink-server/Dockerfile
  - deploy/.env.example
  - deploy/docker-compose.postgres.yml
  - deploy/docker-compose.yml
  - docs/server-deployment.zh-TW.md
  - package-lock.json
-->
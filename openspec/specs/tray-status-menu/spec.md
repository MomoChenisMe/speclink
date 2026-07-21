# tray-status-menu Specification

## Purpose

TBD - created by archiving change 'system-tray-status'. Update Purpose after archive.

## Requirements

### Requirement: 系統匣圖示與原生選單
桌面 app SHALL 於系統匣（macOS 選單列、Windows 通知區域、Linux AppIndicator）顯示 Speclink 圖示；互動樣式 SHALL 由平台決定——macOS 一律為面板（見「面板樣式（macOS）」需求）、非 macOS 平台一律為原生下拉選單（menu-first），SHALL NOT 提供樣式偏好或切換介面。圖示 SHALL 為單色 Speclink 標記（使用者提供的可辨識剪影）；macOS 上 SHALL 以 template 單色形式渲染以適應深淺色選單列。原生選單 SHALL 依序包含：專案區（已開啟的專案分頁）、生命週期分區（提案中／進行中／已就緒）、討論區、動作區（「開啟 Speclink」「設定」「結束」）。

#### Scenario: 非 macOS 啟動後系統匣出現圖示與完整選單
- **WHEN** 使用者於 Windows 或 Linux 啟動桌面 app 且載入完成
- **THEN** 系統匣出現單色 Speclink 標記圖示，展開選單依序可見專案區、生命週期分區、討論區、動作區（「開啟 Speclink」「設定」「結束」）

#### Scenario: macOS 啟動後點擊圖示即面板
- **WHEN** 使用者於 macOS 啟動桌面 app 且載入完成，點擊系統匣圖示
- **THEN** 彈出面板而非原生下拉選單，過程中無需任何樣式設定

#### Scenario: 無專案分頁時選單仍可用
- **WHEN** app 尚未開啟任何專案
- **THEN** 選單不顯示專案區與前導分隔線，其餘區段照常可用


<!-- @trace
source: tray-right-click
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/tray.ts
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
選單主體 SHALL 依生命週期階段分區（提案中、進行中、已就緒）：每個非空階段 SHALL 有一個分區標題，其下 SHALL 列出該階段的所有變更。每張變更列 SHALL 顯示其名稱與任務數「n/m」；有任務（總數大於 0）的變更 SHALL 另顯示一條文字進度條反映完成比例。全無變更時 SHALL 顯示明確空狀態文字。本需求的「非空階段才有分區標題」與「全無變更顯示空狀態文字」SHALL 僅約束原生選單（非 macOS 平台，及 macOS 面板建立失敗的後備）；macOS 面板的生命週期分區呈現 SHALL 依「面板樣式（macOS）」需求——三階段分區常駐、零筆階段以計數 0 的空狀態卡呈現。

#### Scenario: 進行中變更顯示進度條與任務數
- **WHEN** 作用中專案有一個進行中變更、完成 3 個共 12 個任務
- **THEN** 「進行中」分區標題下該變更顯示其名稱、文字進度條與「3/12」

#### Scenario: 全無變更時原生選單顯示空狀態
- **WHEN** 作用中專案沒有任何變更，且互動樣式為原生選單（非 macOS 平台，或 macOS 面板建立失敗後備）
- **THEN** 選單顯示明確空狀態文字而非空白、不出現空階段的分區標題，其餘區段照常


<!-- @trace
source: tray-empty-stage-sections
updated: 2026-07-17
code:
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
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

選單專案區 SHALL 列出全部已開啟的專案分頁。ready 專案 SHALL 以 check item 呈現且作用中專案帶勾選；restoring remote 專案 SHALL 呈現不可重複觸發的「正在連線」狀態；error 或 needs-reauth remote 專案 SHALL 呈現含 workspace 名稱與繁中狀態摘要的復原 submenu，並於 submenu 內標示該專案是否作用中。點選非作用中的 ready 專案 SHALL 使桌面 app 切換至該專案（看板與選單一致更新），且 SHALL NOT 將主視窗帶到前景或奪取焦點。

切換與復原動作 SHALL 以分頁的 locator key 識別目標，SHALL NOT 以 root 路徑為切換把手。local 與 remote ready 分頁點選 SHALL 一視同仁完成切換，SHALL NOT 因 remote 分頁無本機路徑而靜默無反應。remote handshake 失敗時，分頁 SHALL 成為作用中復原目的地，原生選單 SHALL 轉為復原 submenu，至少提供重新連線或重新登入、開啟問題詳情與伺服器設定；直接重新連線 SHALL NOT 喚起主視窗，只有使用者明確選取開啟詳情、伺服器設定或重新登入 SHALL 顯示主視窗並取得焦點。local 專案目錄失效 SHALL 沿用主視窗既有分頁錯誤處理且 app SHALL NOT 崩潰。

#### Scenario: 點選非作用中專案完成切換且不奪焦

- **WHEN** 使用者於系統匣選單點選一個非作用中的 ready 專案
- **THEN** 看板切換至該專案、選單作用中標記移至該專案，主視窗的前景與焦點狀態不變

#### Scenario: 點選 remote 專案分頁完成切換

- **WHEN** 已開啟的分頁中含一個 ready remote 專案分頁，使用者於系統匣選單點選該非作用中 remote 專案
- **THEN** 看板切換至該 remote 專案、選單作用中標記移至該專案，主視窗的前景與焦點狀態不變

#### Scenario: remote 切換失敗轉為復原 submenu

- **WHEN** 使用者於原生系統匣選單點選 remote 專案，而 handshake 因 server 不可達而失敗
- **THEN** 該專案成為作用中復原目的地，選單項轉為含「無法連線」摘要、重新連線、開啟問題詳情與伺服器設定的 submenu，app 未崩潰

#### Scenario: 原生選單直接 retry 不奪焦

- **WHEN** 主視窗隱藏或位於背景，使用者於 error workspace submenu 選取重新連線
- **THEN** workspace 轉為 restoring 並重走 handshake，主視窗維持原顯示與焦點狀態；失敗時 submenu 原位更新，成功時回到 ready 專案項

#### Scenario: 原生選單顯式詳情動作取得焦點

- **WHEN** 使用者於 error workspace submenu 選取開啟問題詳情、伺服器設定或重新登入
- **THEN** 主視窗顯示並取得焦點，分別開啟該 workspace 復原頁、對應 server 設定或對應 connection 登入流程


<!-- @trace
source: remote-workspace-recovery-ux
updated: 2026-07-21
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/src/tray.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/tray_menu.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/RemoteWorkspaceRecovery.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
-->

---
### Requirement: 開啟視窗與結束動作
選單「開啟 Speclink」SHALL 顯示主視窗並使其取得焦點；「設定」SHALL 顯示主視窗、使其取得焦點並將主頁面切換至設定頁；「結束」SHALL 結束 app——原生選單與 macOS 面板皆同（面板的「結束」SHALL 結束整個 app 行程，SHALL NOT 僅收合面板）。除「開啟 Speclink」「設定」、變更子選單動作與討論項外，選單其他操作 SHALL NOT 改變主視窗的顯示與焦點狀態。

#### Scenario: 自最小化狀態開啟主視窗
- **WHEN** 主視窗處於最小化或被其他視窗遮蔽，使用者點選「開啟 Speclink」
- **THEN** 主視窗顯示於前景並取得焦點

#### Scenario: 設定動作開啟主視窗並跳至設定頁
- **WHEN** 主視窗未在前景且顯示看板頁，使用者點選系統匣動作區的「設定」
- **THEN** 主視窗顯示於前景並取得焦點，主頁面切換至設定頁

#### Scenario: 結束 app
- **WHEN** 使用者點選選單的「結束」
- **THEN** app 行程結束，系統匣圖示消失

#### Scenario: 自面板結束 app
- **WHEN** 使用者於 macOS 面板點擊動作區塊的「結束」
- **THEN** app 行程結束，面板與系統匣圖示一併消失


<!-- @trace
source: tray-right-click
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/tray.ts
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

於 macOS，點擊系統匣圖示 SHALL NOT 顯示原生下拉選單，而 SHALL 於圖示下方彈出貼齊圖示的面板視窗，再次點擊 SHALL 收合——無需任何偏好設定。點擊 SHALL 不分滑鼠按鍵：主鍵（左鍵）與次要鍵（右鍵）點擊圖示 SHALL 完全等價，皆為開閉面板。面板內容 SHALL 與原生選單同源（同一前端 store 投影：專案與復原狀態、生命週期分區的變更與進度、討論——討論比照原生選單分「討論」與「已轉出」兩分區），SHALL NOT 為面板另建第二條資料查詢路徑或狀態機。

專案區 SHALL 呈現為橫向 tab 條：每個 tab SHALL 顯示專案名首字母的圓角方塊 avatar 與專案名；remote tab SHALL 另以圖示與文字可辨識 ready、restoring、offline、needs-reauth 或 error，SHALL NOT 只用顏色表意。作用中 ready 專案的 tab SHALL 以實心主色底＋反白文字標示；作用中 restoring／error 專案 SHALL 維持同等清楚的 selected 狀態且不呈 disabled 外觀。tab 總寬超出面板時 SHALL 可橫向捲動且 SHALL NOT 顯示捲軸。

點擊 tab SHALL 原地切換作用中專案，SHALL NOT 喚起主視窗、SHALL NOT 收合面板；切換 SHALL 以分頁的 locator key 為把手（與「選單專案切換」需求同語意），local 與 remote 專案 tab 一視同仁，SHALL NOT 因分頁無本機路徑而靜默無反應。active remote tab 尚無 session且為 restoring／error 時，面板 SHALL 以一張精簡復原卡取代討論與生命週期分區；復原卡 SHALL 顯示 workspace、server、繁中狀態摘要及重新連線或重新登入、在 Speclink 中查看詳情／設定的動作，SHALL NOT 顯示上一個 workspace 的資料。直接重新連線 SHALL 由面板動作回流主視窗 store、面板保持開啟且 SHALL NOT 喚起主視窗；使用者明確選取詳情、設定或重新登入 SHALL 顯示主視窗並取得焦點。active remote session 為 offline 時，面板 SHALL 顯示 stale 狀態列並保留最後成功的變更與討論內容，等待既有 worker 自動收斂。

tab 條尾端 SHALL 有「加入專案」動作項：點擊 SHALL 先顯示主視窗（含切換至其所在桌面——確保後續對話框於使用者眼前可見）再開啟資料夾選擇器（與主視窗「開啟專案」同語意）——選定即以分頁加入該專案並成為作用中專案，取消則無任何變化。資料夾選擇器等系統原生對話框 SHALL 跟隨系統語言呈現（app SHALL 宣告繁體中文在地化，不得固定英文介面）。

active workspace 為 ready 或已有 session 的 offline 時，面板內容 SHALL 依區塊排列：專案 tab 條之下依序為討論區塊（「討論」分區常駐呈現，其後「已轉出」分區有料才現）、生命週期區塊（提案中→進行中→已就緒）、動作區塊（「開啟 Speclink」「設定」「結束」）。專案 tab 條與討論區塊之間、討論區塊與生命週期區塊之間、生命週期區塊與動作區塊之間 SHALL 各有一條分割線（共三條）；區塊內部（分區卡之間）SHALL NOT 出現分割線。此區塊順序為面板刻意設計；原生選單的區段順序仍依「系統匣圖示與原生選單」需求（生命週期分區在前、討論區在後），不受本段影響。無 session 的 restoring／error 狀態 SHALL 以「tab 條、分割線、復原卡、分割線、動作區塊」排列，不渲染討論與生命週期空卡。

生命週期分區與討論分區 SHALL 各自以半透明圓角卡片容器呈現（面板毛玻璃底 SHALL 可透出），分區標題 SHALL 含主色上色的分區圖示，並 SHALL 顯示該分區的項目計數（徽章樣式與看板欄計數同語彙）。生命週期三個階段分區（提案中／進行中／已就緒）SHALL 常駐呈現：零筆階段 SHALL 以「分區標題＋計數 0」的空狀態卡呈現，SHALL NOT 因該階段無變更而整卡消失；全無變更時 SHALL NOT 顯示佔位卡（原「尚無進行中變更」），而以三張計數 0 的分區卡呈現，分區順序固定為提案中→進行中→已就緒。「已轉出」分區 SHALL 維持有料才現——零筆時 SHALL NOT 呈現（與「討論列表」需求一致）。空狀態卡（討論零筆、生命週期零筆階段）SHALL 維持最小高度、內容垂直置中，不得塌陷成細條。有任務的變更列，其進度條填色 SHALL 依階段套用與看板同源的主色深淺階梯（提案中最淺、進行中次之、已就緒最深）。

變更與討論列 SHALL 於列尾常駐複製鈕（複製內容與原生選單的複製動作一致：變更為 name、討論為 slug）；複製鈕點擊後 SHALL 短暫顯示成功回饋（勾號圖示，與看板複製鈕同模式）後自行復原。點擊變更或討論列本體 SHALL 顯示主視窗並開啟對應詳情。

面板開啟時 SHALL NOT 有任何互動元素自動取得焦點（不得出現系統焦點框）；複製鈕 SHALL NOT 可經 Tab 鍵取得焦點。新增的復原 tab、card 與動作 SHALL 提供語意化 label 與清楚 pointer hit area，但 SHALL NOT 將面板改為會奪取前景 app 焦點的 key window。面板高度 SHALL 自適應內容（隨內容增減貼合，達上限高度後面板內部捲動、不得於內容未超限時出現多餘捲動與空白）。面板開啟 SHALL NOT 奪取目前前景 app 的焦點；面板失焦時 SHALL 自動收合。面板視窗建立失敗時 app SHALL 以原生選單樣式運作（選單實作跨平台保留、兼作 macOS 失敗後備）並於設定頁本機設定簽浮出單行錯誤。

#### Scenario: 面板樣式下點擊圖示彈出貼齊面板

- **WHEN** 使用者於 macOS 點擊系統匣圖示
- **THEN** 圖示下方彈出貼齊圖示的面板，頂部為專案 tab 條，其下依作用中 workspace 狀態呈正常資料分區或復原卡，尾端為動作區塊，未出現原生下拉選單

#### Scenario: 右鍵點擊圖示與左鍵等價

- **WHEN** 使用者於 macOS 以滑鼠次要鍵（右鍵）點擊系統匣圖示
- **THEN** 面板於圖示下方彈出且貼齊位置與左鍵點擊一致；面板已開啟時再以右鍵點擊圖示則面板收合——與左鍵行為完全相同

#### Scenario: ready workspace 的區塊順序與分割線

- **WHEN** 面板開啟，作用中 ready 專案存在討論中討論、已轉出討論與各階段變更
- **THEN** 由上而下依序為：專案 tab 條、分割線、「討論」分區、「已轉出」分區、分割線、「提案中」「進行中」「已就緒」分區、分割線、「開啟 Speclink」「設定」「結束」；分割線恰為三條且僅出現於區塊之間、分區卡之間無分割線

#### Scenario: 點擊專案 tab 原地切換

- **WHEN** 面板開啟且有兩個以上 ready 專案分頁，使用者點擊非作用中專案的 tab
- **THEN** 該 tab 轉為實心主色的作用中標示，面板下方的變更與討論內容切換為該專案，主視窗未被喚起、面板保持開啟

#### Scenario: 點擊 remote 專案 tab 原地切換

- **WHEN** 面板開啟且分頁中含一個非作用中的 ready remote 專案，使用者點擊該 remote 專案的 tab
- **THEN** 該 tab 轉為作用中標示，面板下方的變更與討論內容切換為該 remote 專案，主視窗未被喚起、面板保持開啟

#### Scenario: remote handshake 失敗顯示復原卡

- **WHEN** 面板中點擊一個無 session 的 remote tab，handshake 因 server 不可達而失敗
- **THEN** 該 tab 維持作用中 error 狀態，面板以 workspace／server／繁中摘要與復原動作卡取代討論及生命週期分區，未顯示上一 workspace 資料，主視窗未被喚起且面板保持開啟

#### Scenario: 面板 retry 原地恢復

- **WHEN** 使用者於 error 復原卡選取重新連線且 handshake 成功
- **THEN** tab 先呈 restoring，成功後轉 ready 並恢復該 workspace 的討論與生命週期分區，面板全程保持開啟且主視窗未被喚起

#### Scenario: 面板顯式開啟詳情或重新登入

- **WHEN** 使用者於復原卡選取在 Speclink 中查看詳情、伺服器設定或重新登入
- **THEN** 主視窗顯示並取得焦點，開啟對應 workspace 復原頁、server 設定或 connection 登入流程

#### Scenario: 已建立 session 離線保留 Panel stale 內容

- **WHEN** 作用中 remote session 已載入內容後進入 offline
- **THEN** 面板顯示 offline／stale 狀態列並保留最後成功的討論與生命週期內容，未改為無 session 復原卡

#### Scenario: tab 條尾端快速加入專案

- **WHEN** 主視窗位於另一個桌面或未在前景，使用者點擊 tab 條尾端的「加入專案」項並於資料夾選擇器選定一個專案目錄
- **THEN** 主視窗先被喚起（桌面切換至其所在處）、資料夾選擇器於前景出現；選定後該專案以分頁加入並成為作用中專案；於選擇器按取消則分頁無任何變化

#### Scenario: 分區標題顯示項目計數

- **WHEN** ready workspace 的面板列出提案中 1 筆變更、討論 0 筆
- **THEN** 「提案中」分區標題帶計數徽章 1；討論空狀態卡顯示計數 0 且維持最小高度、內容垂直置中

#### Scenario: 全無變更時三個生命週期分區常駐

- **WHEN** 作用中 ready 專案沒有任何變更，使用者於 macOS 開啟面板
- **THEN** 面板依序呈現「提案中」「進行中」「已就緒」三張分區卡，各帶計數徽章 0、維持最小高度且內容垂直置中，未出現「尚無進行中變更」佔位卡

#### Scenario: 部分有資料時空階段分區仍常駐

- **WHEN** 作用中 ready 專案僅有 1 個進行中變更，無提案中與已就緒變更
- **THEN** 「進行中」分區卡帶計數徽章 1 並列出該變更；「提案中」與「已就緒」分區卡仍呈現且各帶計數徽章 0，三張分區卡依提案中→進行中→已就緒順序排列

#### Scenario: 進度條依階段深淺

- **WHEN** ready workspace 的面板同時列出提案中與進行中各一個有任務的變更
- **THEN** 兩列進度條填色同為主色但深淺不同——提案中較淺、進行中較深，與看板欄位的階段配色同階梯

#### Scenario: 開啟面板無預設焦點

- **WHEN** 使用者點擊系統匣圖示開啟面板
- **THEN** 面板中無任何元素帶系統焦點框（含第一顆複製鈕或復原動作），可點擊動作仍有語意 label 與可見回饋

#### Scenario: 面板不搶焦點且失焦自動收合

- **WHEN** 使用者於其他 app 位於前景時點擊系統匣圖示開啟面板，隨後點擊面板外任意處
- **THEN** 面板開啟期間原前景 app 保持焦點；點擊面板外後面板自動收合

#### Scenario: 面板內以常駐複製鈕複製

- **WHEN** 使用者點擊面板中某討論列列尾的複製鈕（無需 hover 顯示、常駐可見）
- **THEN** 系統剪貼簿內容等於該討論的 slug，複製鈕短暫轉為勾號回饋後復原，面板保持開啟、未開啟主視窗

#### Scenario: 面板高度自適應內容

- **WHEN** 面板開啟且正常內容或復原卡高度少於一屏
- **THEN** 面板高度貼合內容（下方無大片空白），內容增加超過上限高度後面板內部出現捲動

#### Scenario: 面板建立失敗退回原生選單

- **WHEN** macOS 上面板視窗建立失敗
- **THEN** 系統匣以原生選單樣式運作，remote error workspace 仍可經復原 submenu 操作，設定頁本機設定簽浮出單行面板建立錯誤


<!-- @trace
source: remote-workspace-recovery-ux
updated: 2026-07-21
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/src/tray.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/tray_menu.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/RemoteWorkspaceRecovery.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tray.ts
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
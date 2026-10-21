# tray-status-menu Specification

## Purpose

桌面 app 的系統匣狀態選單：原生選單依生命週期分區呈現變更與進度、變更子選單的快捷動作、討論列表、專案切換與開啟視窗／結束，以及 macOS 的面板樣式與分區溢出摺疊。本 capability 保證選單內容與看板同源、不另建一條資料路徑，並在面板的變更列上帶出品質關卡蓋章狀態。

## Requirements

### Requirement: 系統匣圖示與原生選單

桌面 app SHALL 於系統匣（macOS 選單列、Windows 通知區域、Linux AppIndicator）顯示 Speclink 圖示；互動樣式 SHALL 由平台決定——macOS 為面板（主鍵與次要鍵皆開閉面板）、Windows 為面板加原生選單（主鍵開閉面板、次要鍵顯示原生選單）、Linux 一律為原生下拉選單（menu-first）；面板依「面板樣式」需求，SHALL NOT 提供樣式偏好或切換介面。圖示 SHALL 為單色 Speclink 標記（使用者提供的可辨識剪影）；macOS 上 SHALL 以 template 單色形式渲染以適應深淺色選單列。原生選單 SHALL 依序包含：專案區（已開啟的專案分頁）、生命週期分區（提案中／進行中／已就緒）、討論區、動作區（「開啟 Speclink」「專案設定」「設定」「結束」）。

#### Scenario: 非 macOS 啟動後系統匣出現圖示與完整選單
- **WHEN** 使用者於 Linux 啟動桌面 app 且載入完成後點擊系統匣圖示，或於 Windows 啟動後以次要鍵點擊系統匣圖示
- **THEN** 系統匣出現單色 Speclink 標記圖示，展開選單依序可見專案區、生命週期分區、討論區、動作區（「開啟 Speclink」「專案設定」「設定」「結束」）

#### Scenario: macOS 啟動後點擊圖示即面板
- **WHEN** 使用者於 macOS 啟動桌面 app 且載入完成，點擊系統匣圖示
- **THEN** 彈出面板而非原生下拉選單，過程中無需任何樣式設定

#### Scenario: Windows 主鍵點擊圖示即面板
- **WHEN** 使用者於 Windows 啟動桌面 app 且載入完成，以主鍵點擊系統匣圖示
- **THEN** 彈出面板而非原生下拉選單，過程中無需任何樣式設定

#### Scenario: 無專案分頁時選單仍可用
- **WHEN** app 尚未開啟任何專案
- **THEN** 選單不顯示專案區與前導分隔線，其餘區段照常可用


<!-- @trace
source: tray-panel-windows
updated: 2026-10-13T17:38:06+08:00
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

選單主體 SHALL 依生命週期階段分區（提案中、進行中、已就緒）：每個非空階段 SHALL 有一個分區標題，其下 SHALL 列出該階段的所有變更。每張變更列 SHALL 顯示其名稱與任務數「n/m」；有任務（總數大於 0）的變更 SHALL 另顯示一條文字進度條反映完成比例。全無變更時 SHALL 顯示明確空狀態文字。本需求的「非空階段才有分區標題」與「全無變更顯示空狀態文字」SHALL 僅約束原生選單（Linux 的系統匣選單、Windows 的右鍵選單，及面板建立失敗的後備）；面板（macOS 與 Windows）的生命週期分區呈現 SHALL 依「面板樣式」需求——三階段分區常駐、零筆階段以計數 0 的空狀態卡呈現。

#### Scenario: 進行中變更顯示進度條與任務數
- **WHEN** 作用中專案有一個進行中變更、完成 3 個共 12 個任務
- **THEN** 「進行中」分區標題下該變更顯示其名稱、文字進度條與「3/12」

#### Scenario: 全無變更時原生選單顯示空狀態
- **WHEN** 作用中專案沒有任何變更，且使用者開啟的是原生選單（Linux 的系統匣選單、Windows 的右鍵選單，或面板建立失敗後備）
- **THEN** 選單顯示明確空狀態文字而非空白、不出現空階段的分區標題，其餘區段照常


<!-- @trace
source: tray-panel-windows
updated: 2026-10-13T17:38:06+08:00
-->

---
### Requirement: 變更子選單動作
每張變更 SHALL 呈現為子選單，其中 SHALL 依序含「開啟此變更」與「複製名稱」動作。選取「開啟此變更」SHALL 顯示主視窗並取得焦點、且開啟該變更的詳情頁（主區切至看板並換成該頁，互斥與浮層收掉規則依 desktop-app「詳情頁互斥」）。選取「複製名稱」SHALL 將該變更的 name（純文字，不含進度條字元與任務數）寫入系統剪貼簿，且主視窗隱藏或無焦點時 SHALL 仍寫入成功；剪貼簿寫入失敗時 SHALL NOT 彈出錯誤視窗、SHALL NOT 中斷選單操作。

#### Scenario: 開啟某變更詳情
- **WHEN** 使用者展開某變更子選單並選取「開啟此變更」
- **THEN** 主視窗顯示於前景並取得焦點，主區為該變更的詳情頁

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
source: desktop-detail-page
updated: 2026-10-10T15:55:28+08:00
-->

---
### Requirement: 討論列表

選單討論區 SHALL 分兩個分區呈現 active 討論：「討論」分區列引擎不會自動收走的討論——含討論中（尚未轉出變更）、已轉出但尚無結論（promoted 且 concluded 為 false）、以及已轉出且已有結論但保留在途（promoted 且 concluded 為 true 且 hold 為 true）者；其後「已轉出」分區列已轉出且已有結論且未保留（promoted 且 concluded 為 true 且 hold 非 true）的討論，無此類討論時 SHALL NOT 顯示該分區。分區判準 SHALL 由快照導出的單一布林承載，面板與原生選單 SHALL NOT 各自計算。面板 SHALL 以獨立的「已轉出」卡呈現已轉出討論，置於「討論」卡正下方，SHALL NOT 收於「討論」卡之內：卡標頭依序為中性灰的討論圖示（與「討論」卡同款）、「已轉出」、中性灰計數徽章（N＝已轉出筆數），SHALL NOT 使用頁尾「開啟 Speclink」的箭頭圖示；卡內的已轉出討論列 SHALL 與討論中列同構（slug 為題、topic 為描述、列尾複製鈕）、同受「分區溢出摺疊」約束，SHALL NOT 另加底色。已轉出卡 SHALL 預設收合，收合觸發、箭頭方向與收合外觀依「面板內容卡收合」需求；展開狀態 SHALL 存於 app 本機（與看板討論欄的記憶分開存），跨面板開閉與 app 重啟保留；記憶缺席或壞值 SHALL 視為收合。零筆已轉出時 SHALL NOT 呈現已轉出卡；討論清單載入中或載入失敗時（已轉出筆數未知）SHALL NOT 呈現已轉出卡。「討論」卡的計數與空狀態 SHALL 只依討論中筆數：討論中為零而已轉出非零時，「討論」卡 SHALL 為計數 0 的空狀態卡，其下接已轉出卡。討論清單資料的 concluded 欄位缺席（向後相容路徑）時 SHALL 退回既有判準——promoted 一律列於「已轉出」分區；hold 欄位缺席時 SHALL 視同未保留、SHALL NOT 把缺席當成 true。兩分區的每則討論 SHALL 呈現為子選單：父項標籤 SHALL 為該討論的 slug（識別錨點直出）；子選單 SHALL 依序含 topic 描述行（disabled、不可選取）、「開啟此討論」、「複製 slug」；保留在途的討論 SHALL NOT 帶額外標籤。選取「開啟此討論」SHALL 顯示主視窗並取得焦點、且開啟該討論的詳情頁（主區切至看板並換成該頁）。選取「複製 slug」SHALL 將該討論的 slug 寫入系統剪貼簿，主視窗隱藏或無焦點時 SHALL 仍寫入成功。無討論中討論時「討論」分區 SHALL 顯示「討論 0」。

面板已轉出卡的展開內容 SHALL 落在該卡標頭下方，箭頭 SHALL 與其他內容卡相同（展開朝下、收合朝右），SHALL NOT 沿用舊收合列「展開時朝上」的規則。

#### Scenario: 討論以 slug 為題、topic 為描述

- **WHEN** 存在 active 討論（slug 為 board-search-bar、topic 為「看板搜尋列」）
- **THEN** 討論區該項父標籤為「board-search-bar」，展開子選單首行為灰字「看板搜尋列」且不可選取

#### Scenario: 開啟某討論

- **WHEN** 使用者展開某討論子選單並選取「開啟此討論」
- **THEN** 主視窗顯示於前景並取得焦點，主區為該討論的詳情頁

#### Scenario: 複製討論 slug

- **WHEN** 使用者展開某討論子選單並選取「複製 slug」
- **THEN** 系統剪貼簿內容等於該討論的 slug

#### Scenario: 已轉出討論列於已轉出分區

- **WHEN** 存在討論中的討論與已轉出且已有結論且無 hold 的討論各一
- **THEN** 討論中的討論列於「討論」分區、已轉出且已有結論且無 hold 的列於「已轉出」分區（原生選單）或「討論」卡下方「已轉出」卡（計數 1）展開後的列（面板），兩者子選單／列結構相同（topic 描述行、開啟此討論、複製 slug）

#### Scenario: 已轉出但尚無結論的討論列於討論分區

- **WHEN** 存在一筆已轉出但尚無結論（promoted 且 concluded 為 false）的討論
- **THEN** 該討論列於「討論」分區而非「已轉出」分區；若無其他已轉出且已有結論的討論，原生選單不出現「已轉出」分區標題、面板不出現「已轉出」卡

#### Scenario: 保留中的已轉出討論列於討論分區

- **WHEN** 存在一筆已轉出、已有結論且 hold 為 true 的討論，與一筆已轉出、已有結論且 hold 為 false 的討論
- **THEN** 前者列於「討論」分區並計入「討論 N」、子選單結構與其他討論相同且無額外標籤；後者列於「已轉出」分區

#### Scenario: 無已轉出討論時不顯示該分區

- **WHEN** 目前沒有任何已轉出且已有結論且無 hold 的討論
- **THEN** 原生選單不出現「已轉出」分區標題，面板不出現「已轉出」卡

#### Scenario: 面板的已轉出卡預設收合並記憶展開

- **WHEN** 面板開啟，作用中專案有 2 則討論中討論與 3 則已轉出討論，沒有已轉出卡的展開記憶；使用者點按「已轉出」卡標頭，之後關閉面板再重新開啟
- **THEN** 首次開啟時「討論」卡只列 2 則討論中討論，其下以卡片間距隔開的「已轉出」卡只有標頭一行（中性灰圖示、「已轉出」、計數 3、箭頭朝右）；點按後 3 則已轉出討論列於該卡標頭下方、箭頭朝下；再點按收合後箭頭恢復朝右；重新開啟面板後仍為展開狀態，app 重啟後亦然

#### Scenario: 面板已轉出展開區受溢出摺疊約束

- **WHEN** 已轉出討論有 7 則且已轉出卡已展開
- **THEN** 已轉出卡直列前 5 則、尾端為「還有 2 個…」可展開列，與其他分區同規則

#### Scenario: 已轉出卡與討論卡以間距分開

- **WHEN** 作用中專案有 11 則討論中討論與 3 則已轉出討論，「討論」卡與「已轉出」卡皆展開
- **THEN** 「討論」卡直列前 5 則、最後一列為「還有 6 個…」；「已轉出」卡為另一張白底細框卡，與「討論」卡之間只有卡片間距；已轉出列沒有額外底色，卡標頭的圖示與計數徽章為中性灰

#### Scenario: 討論中為零時討論卡為空狀態卡

- **WHEN** 作用中專案沒有討論中的討論，有 2 則已轉出討論
- **THEN** 「討論」卡為計數 0 的空狀態卡（維持最小高度、內容垂直置中），其下為計數 2 的「已轉出」卡

#### Scenario: 載入中與載入失敗不呈現已轉出卡

- **WHEN** 面板的討論清單仍在載入中，或載入失敗
- **THEN** 面板不呈現「已轉出」卡；「討論」卡依載入狀態呈現佔位列或失敗提示

#### Scenario: 無討論時顯示零

- **WHEN** 目前沒有任何討論中的討論
- **THEN** 「討論」分區顯示「討論 0」


<!-- @trace
source: tray-promoted-separation
updated: 2026-10-14T23:04:04+08:00
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
選單「開啟 Speclink」SHALL 顯示主視窗並使其取得焦點；「專案設定」SHALL 顯示主視窗、使其取得焦點並將主頁面切換至專案設定頁（呈現作用中專案的專案層設定；作用中 workspace 處於復原或 stale 狀態時 SHALL 照常可用，SHALL NOT 特判隱藏或停用——導覽後的呈現交由主視窗既有行為）；「設定」SHALL 顯示主視窗、使其取得焦點並將主頁面切換至設定頁；「結束」SHALL 結束 app——原生選單與 macOS 面板皆同（面板的「結束」SHALL 結束整個 app 行程，SHALL NOT 僅收合面板）。動作區順序 SHALL 為「開啟 Speclink」「專案設定」「設定」「結束」（專案層動作在前、app 層在後，與主視窗側欄層次一致）；「專案設定」文案 SHALL 與主視窗側欄「專案設定」詞條一致，macOS 面板該列圖示 SHALL 沿用側欄同款。除「開啟 Speclink」「專案設定」「設定」、變更子選單動作與討論項外，選單其他操作 SHALL NOT 改變主視窗的顯示與焦點狀態。

#### Scenario: 自最小化狀態開啟主視窗
- **WHEN** 主視窗處於最小化或被其他視窗遮蔽，使用者點選「開啟 Speclink」
- **THEN** 主視窗顯示於前景並取得焦點

#### Scenario: 專案設定動作開啟主視窗並跳至專案設定頁
- **WHEN** 主視窗未在前景且顯示看板頁，使用者點選系統匣動作區的「專案設定」
- **THEN** 主視窗顯示於前景並取得焦點，主頁面切換至作用中專案的專案設定頁

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
source: tray-panel-scroll-and-vibrancy
updated: 2026-07-31
code:
  - apps/desktop/panel.html
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/panel/main.tsx
  - apps/desktop/src/tray.ts
  - apps/desktop/vitest.setup.ts
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

---
### Requirement: 面板變更列的品質站章

macOS 面板的變更列 SHALL 依協定的審查與驗證狀態，於名稱與任務數之間並排渲染行內站章，順序固定（審查章在前、驗證章在後）：`none` 無章；`inReview`／`inVerify` 顯示進行中章；`reviewed`／`verified` 顯示已蓋章章；`reviewedStale`／`verifiedStale` 顯示降級樣式章。各章的圖示、色調與 tooltip 詞條 SHALL 與看板卡片的對應章共用同一組樣式與 i18n 詞條，SHALL NOT 另建第二份對照。變更列 hover（半透明前景色灰底）期間，站章 SHALL 維持共用色調、SHALL NOT 改色——列不以主色實心反白，章的對比不受 hover 影響。變更列 SHALL NOT 渲染看板卡片的其他行內符號（建立者頭像、來源討論標記、restale 標記、metaError 標記）。原生選單（非 macOS 平台，及 macOS 面板建立失敗的後備）的變更列標籤 SHALL 維持既有「名稱＋文字進度條＋任務數」組成、SHALL NOT 加入站章。

#### Scenario: 面板兩章並排

- **WHEN** 面板開啟且某變更的協定資料為 `reviewStatus: "reviewed"` 且 `verifyStatus: "verified"`
- **THEN** 該變更列於名稱與任務數之間依序顯示審查章與驗證章，tooltip 分別為「已審查」與「已驗證」（tw）

#### Scenario: hover 灰底時章色調不變

- **WHEN** 指標停留於帶兩站章的變更列，列呈半透明前景色灰底
- **THEN** 兩站章維持與看板卡片共用的色調、未改色，站別由圖示與色調同時可辨

#### Scenario: 僅驗證章

- **WHEN** 某變更的協定資料為 `reviewStatus: "none"` 且 `verifyStatus: "inVerify"`
- **THEN** 該變更列僅顯示「驗證中」驗證章，無審查章

#### Scenario: 無章時列組成不變

- **WHEN** 某變更的兩站狀態皆為 `none`
- **THEN** 該變更列不渲染任何站章與其他新增元素，列組成與本變更前一致

#### Scenario: 原生選單不受影響

- **WHEN** 於原生選單樣式（非 macOS 平台，或 macOS 面板建立失敗後備）展開系統匣選單，某變更為 `reviewStatus: "reviewed"`
- **THEN** 該變更列標籤仍為「名稱＋文字進度條＋任務數」，不含任何站章字元


<!-- @trace
source: tray-panel-restyle
updated: 2026-10-12T10:39:02+08:00
-->

---
### Requirement: 面板分頁切換中回饋

tray 面板分頁條在本地分頁的專案探測期間 SHALL 於目標分頁顯示切換中 spinner，分區內容 SHALL 維持原 workspace 資料照常呈現；探測成功或失敗時 spinner SHALL 消失。切換中狀態 SHALL 經 TraySnapshot 自主視窗 store 導出，面板 SHALL NOT 自建切換中狀態。

#### Scenario: 面板點擊分頁立即回饋

- **WHEN** 於 tray 面板點擊非活躍的本地分頁且專案探測尚未完成
- **THEN** 目標分頁立即顯示 spinner，分區維持原 workspace 內容

#### Scenario: 切換完成後高亮更新

- **WHEN** 專案探測成功且活躍分頁切換完成
- **THEN** spinner 消失，面板分頁高亮移至新活躍分頁

<!-- @trace
source: desktop-loading-skeleton-ux
updated: 2026-08-12
-->

---
### Requirement: 面板分區首訪 skeleton

活躍 workspace 的快照尚未完成首次載入時，tray 面板各分區 SHALL 以 skeleton 佔位列呈現內容、分區標題 SHALL 照常顯示；快照已載入（含舊快取）時 SHALL 直接顯示既有資料，重載完成靜默更新。載入中狀態 SHALL 經 TraySnapshot 導出。remote 復原態既有的資料遮蔽 SHALL 優先於 skeleton——遮蔽期間不顯示骨架。

#### Scenario: 首訪 workspace 分區顯示骨架列

- **WHEN** 切換至首次造訪的 workspace 且整批載入尚未完成
- **THEN** 面板分區標題照常，內容為 skeleton 佔位列，與「尚無進行中變更」空態可區分

#### Scenario: 已訪 workspace 直接顯示舊資料

- **WHEN** 切換至先前造訪過的 workspace
- **THEN** 分區立即顯示上次快照內容，全程無 skeleton

#### Scenario: remote 復原態不出骨架

- **WHEN** 活躍分頁為 remote 且處於遮蔽 workspace 資料的復原狀態
- **THEN** 面板顯示既有復原呈現，不顯示 skeleton

<!-- @trace
source: desktop-loading-skeleton-ux
updated: 2026-08-12
-->

---
### Requirement: 面板分區載入失敗終態

活躍 workspace 首訪整批載入失敗時，tray 面板各分區 SHALL 顯示載入失敗提示、SHALL NOT 與「尚無進行中變更」等空態同貌、SHALL NOT 停留在 skeleton；載入態的翻轉 SHALL 即時抵達面板（不因清單內容去抖而延遲骨架收掉）。失敗終態 SHALL 經 TraySnapshot 自主視窗 store 導出，面板 SHALL NOT 自建失敗狀態。remote 復原態既有的資料遮蔽 SHALL 維持優先——遮蔽期間不顯示失敗提示。

#### Scenario: 面板首訪失敗顯示提示

- **WHEN** 活躍 workspace 首訪整批載入失敗
- **THEN** 面板分區骨架即時收掉，內容顯示載入失敗提示，與空態列可區分

#### Scenario: 復原遮蔽優先於失敗提示

- **WHEN** 活躍分頁為 remote 且處於遮蔽 workspace 資料的復原狀態
- **THEN** 面板顯示既有復原呈現，不顯示失敗提示與 skeleton

<!-- @trace
source: desktop-refreshing-inflight-set
updated: 2026-08-12
-->

---
### Requirement: 變更列的波次標示與順序同源

系統匣的變更列順序 SHALL 與看板同一份清單 payload 的順序一致（即 plan 配置順序投影到各階段），SHALL NOT 另行排序。macOS 面板的變更列 SHALL 於清單項帶 `wave` 時在名稱之前渲染波次數字（與看板卡片波次章共用同一 i18n tooltip 詞條「第 N 波」）；`blockedBy` 非空的列 SHALL 整列降透明度且 tooltip 列出前置名稱；列 hover（半透明前景色灰底）期間數字 SHALL 維持主色、SHALL NOT 改色。原生選單（非 macOS 平台，及 macOS 面板建立失敗的後備）的變更列標籤 SHALL 於 `wave` 存在時前綴「N· 」，SHALL NOT 做變淡或 tooltip。清單項缺 `wave`（remote 或 plan 成環）時面板與原生選單的列 SHALL 與本需求引入前一致。

#### Scenario: 面板列首波次與被擋變淡

- **WHEN** 面板開啟，進行中分區有 wave=1 的 add-a 與 wave=2、blockedBy=["add-a"] 的 add-b
- **THEN** 兩列依 add-a、add-b 順序，列首分別為「1」「2」，add-b 整列降透明度且 tooltip 為「等待：add-a」

#### Scenario: 原生選單前綴數字

- **WHEN** 於原生選單樣式展開系統匣，某變更 wave=3
- **THEN** 該變更列標籤為「3· 名稱＋文字進度條＋任務數」

#### Scenario: 缺欄位不變

- **WHEN** remote 專案的面板開啟（清單項無 wave）
- **THEN** 變更列無波次數字、不變淡，與本需求引入前一致


<!-- @trace
source: tray-panel-restyle
updated: 2026-10-12T10:39:02+08:00
-->

---
### Requirement: 面板交接主視窗後滑鼠互動完整

於 macOS，使用者自面板觸發任一會顯示主視窗的動作（點擊變更列、討論列、「開啟 Speclink」、專案設定、設定、復原卡的詳情／伺服器設定／重新登入、加入專案）後，主視窗 SHALL 於顯示到前景的同時具備完整的滑鼠互動：滑鼠移到任一互動元素上 SHALL 觸發其 hover 樣式，並呈現該元素依 desktop-app「可點元素的游標與 hover 回饋」應有的游標（可點元素為箭頭、文字輸入為文字游標），具 tooltip 的元素 SHALL 顯示 tooltip；SHALL NOT 出現「點擊可操作但 hover、游標樣式、tooltip 皆無反應」的狀態。此保證 SHALL 不依賴使用者再切換 app、調整視窗尺寸或等待。

交接 SHALL 先收合面板再顯示主視窗；面板已因失焦先行收合時，交接 SHALL 仍成功、SHALL NOT 重新打開面板。既有「面板不搶焦點且失焦自動收合」與 Cmd-Tab 回前景的行為 SHALL 維持不變。

#### Scenario: 自面板點變更列回前景後詳情頁內 hover 正常

- **WHEN** 主視窗在背景至少 5 秒，使用者點擊系統匣圖示開啟面板、點擊其中一個變更列，主視窗顯示到前景並開啟該變更的詳情頁
- **THEN** 滑鼠移到詳情頁內的按鈕上，該按鈕顯示 hover 樣式且游標為箭頭

#### Scenario: 關閉詳情頁後看板卡片 hover 與 tooltip 正常

- **WHEN** 承上一情境，使用者按 Esc 或點麵包屑回到看板，滑鼠移到看板任一卡片上
- **THEN** 卡片顯示 hover 樣式，卡片上具 tooltip 的元素顯示 tooltip

#### Scenario: 自面板「開啟 Speclink」與討論列回前景同樣正常

- **WHEN** 主視窗在背景至少 5 秒，使用者開啟面板後點擊「開啟 Speclink」或任一討論列
- **THEN** 主視窗顯示到前景後，滑鼠移到任一可點元素上即有 hover 樣式且游標為箭頭

#### Scenario: 面板已先收合時交接仍成功

- **WHEN** 使用者點擊面板中的變更列，而面板在主視窗顯示前已因失焦自行收合
- **THEN** 主視窗仍顯示到前景並開啟該變更詳情，面板 SHALL NOT 重新出現，主視窗滑鼠互動完整

#### Scenario: Cmd-Tab 回前景行為不變

- **WHEN** 使用者未經面板、以 Cmd-Tab 將主視窗切回前景
- **THEN** 主視窗取得焦點且滑鼠互動完整，與本需求加入前相同


<!-- @trace
source: clickable-cursor-hover-rule
updated: 2026-10-13T11:49:40+08:00
-->

---
### Requirement: 面板內容卡收合

面板的內容卡（提案中、進行中、已就緒、討論，以及有料時呈現的已轉出）SHALL 各自可收合。整條分區標題 SHALL 為收合切換的觸發區，計數徽章之後 SHALL 有方向箭頭（展開朝下、收合朝右），觸發區 SHALL 以語意化的展開狀態（aria-expanded）反映目前狀態，SHALL NOT 可經 Tab 鍵取得焦點。收合的卡 SHALL 只呈現分區標題一行（圖示、名稱、計數、箭頭），SHALL NOT 呈現其下的列、骨架、失敗提示與「還有 N 個…」溢出列，SHALL NOT 套用空狀態卡的最小高度，標題下 SHALL NOT 有細線；計數 SHALL 照常隨資料更新（載入中不出計數，與展開時相同）。收合狀態 SHALL 只由使用者點擊改變：收合中的卡有項目增減時 SHALL NOT 自動展開。卡重新展開時，「還有 N 個…」溢出列 SHALL 回到未展開。各卡的收合 SHALL 互不牽動：收起或展開「討論」卡時，已轉出卡 SHALL 照常呈現並維持其自身的收合狀態，反之亦然。收合與展開後面板高度 SHALL 依面板的高度自適應規定重新貼合內容。

沒有收合記憶時，提案中、進行中、已就緒、討論四張卡 SHALL 全部展開，已轉出卡 SHALL 收合（其展開狀態另存，依「討論列表」需求）。收合狀態 SHALL 記憶於 app 本機、所有專案共用一份：切換作用中專案、關閉再開啟面板、重啟 app 後 SHALL 維持；記憶讀取失敗或值無法解析時，四張卡 SHALL 視為全部展開、已轉出卡 SHALL 視為收合，寫入失敗 SHALL 靜默，皆 SHALL NOT 顯示錯誤。專案 tab 條卡、動作區塊卡、復原卡與 stale 狀態列 SHALL NOT 可收合。系統匣原生選單 SHALL NOT 受本需求影響。

#### Scenario: 點分區標題收合與展開

- **WHEN** 面板列出進行中 3 筆變更，使用者點擊「進行中」分區標題，之後再點一次
- **THEN** 第一次點擊後進行中卡只剩標題一行（圖示、「進行中」、計數 3、朝右箭頭），三列變更不再呈現、面板高度隨之縮短，觸發區的展開狀態為收合；第二次點擊後三列變更恢復、箭頭朝下、展開狀態為展開

#### Scenario: 沒有收合記憶時四張卡展開、已轉出卡收合

- **WHEN** 面板沒有收合記憶（首次使用，或記憶值無法解析），作用中專案有 2 則已轉出討論，使用者開啟面板
- **THEN** 提案中、進行中、已就緒、討論四張卡全部展開，「已轉出」卡只有標頭一行（計數 2、箭頭朝右），面板未顯示任何錯誤

#### Scenario: 收合狀態跨專案與重開保留

- **WHEN** 使用者在專案 A 收起「討論」卡，關閉面板再開啟，再切到專案 B，之後重啟 app 再開啟面板
- **THEN** 每次開啟與切換後討論卡都只剩標題一行，計數為當下作用中專案的討論數

##### Example: 收起討論後切換專案

- **GIVEN** wadpilot 有 8 則討論、speclink 有 2 則討論，面板作用中專案為 wadpilot
- **WHEN** 使用者收起「討論」卡後點擊 speclink 的專案 tab
- **THEN** 討論卡只剩一行「討論」與計數 2，箭頭朝右

#### Scenario: 收合中的卡有新項目時不自動展開

- **WHEN** 「已就緒」卡收合且計數 0，一個進行中變更的任務全數完成而進入已就緒
- **THEN** 已就緒卡維持收合，標題計數由 0 變 1

#### Scenario: 收合的空狀態卡只剩一行

- **WHEN** 「已就緒」計數 0 時使用者收起已就緒卡
- **THEN** 已就緒卡只剩標題一行，不保留空狀態卡的最小高度，標題下沒有細線

#### Scenario: 重新展開時溢出列回到未展開

- **WHEN** 討論卡有 8 則討論中討論、已轉出卡有 3 則已轉出討論，使用者展開「還有 3 個…」與已轉出卡，之後收起討論卡再展開
- **THEN** 討論卡重新展開後直列前 5 則與未展開的「還有 3 個…」列；已轉出卡全程照常呈現且維持展開

#### Scenario: 收起討論卡時已轉出卡照常呈現

- **WHEN** 「討論」卡展開、「已轉出」卡收合且計數 3，使用者點擊「討論」分區標題
- **THEN** 「討論」卡只剩標題一行；「已轉出」卡仍在其下呈現標頭一行（「已轉出」、計數 3、箭頭朝右），收合狀態不變

#### Scenario: 固定頁首頁尾與復原卡不可收合

- **WHEN** 面板呈現專案 tab 條卡、動作區塊卡，或 active remote 專案的復原卡與 stale 狀態列
- **THEN** 這些卡與狀態列沒有收合箭頭，點擊其空白處不會收起任何內容

#### Scenario: 原生選單不受收合影響

- **WHEN** 使用者於非 macOS 平台（Windows 或 Linux）開啟系統匣原生選單，作用中專案有 2 則討論與 1 個進行中變更
- **THEN** 原生選單的討論區與生命週期分區照舊列出項目，沒有收合控制


<!-- @trace
source: tray-promoted-separation
updated: 2026-10-14T23:04:04+08:00
-->

---
### Requirement: 面板樣式

於 macOS 與 Windows，點擊系統匣圖示 SHALL 彈出面板視窗（觸發的滑鼠按鍵、彈出位置與焦點行為依本需求的平台差異段），再次點擊 SHALL 收合——無需任何偏好設定。面板內容 SHALL 與原生選單同源（同一前端 store 投影：專案與復原狀態、生命週期分區的變更與進度、討論——「討論」與「已轉出」的分流判準與原生選單同源，面板的已轉出以討論卡下方的獨立卡呈現，依「討論列表」需求），SHALL NOT 為面板另建第二條資料查詢路徑或狀態機。兩平台的面板 SHALL 為同一份呈現層：本需求平台差異段以外的規定，以及本規格其他需求中針對面板的呈現與動作（含寫作「macOS 面板」者），兩平台 SHALL 一體適用；唯「面板交接主視窗後滑鼠互動完整」仍僅限 macOS。Linux 不提供面板（依「系統匣圖示與原生選單」需求）。

面板內容 SHALL 分成三類白底（--card token）細框 12px 圓角卡疊於面板底上：專案 tab 條一張、中段每個分區各一張（復原卡與 stale 狀態列亦為卡）、動作區塊一張；卡與卡之間以間距分隔，SHALL NOT 以分割線分隔區塊。

專案區 SHALL 呈現為橫向 tab 條（一張卡）：每個 tab SHALL 顯示專案名首字母的圓角方塊與其下的專案名；方塊的尺寸、圓角與配色 SHALL 與主視窗圖示列的專案方塊同形（取同一份樣式常數）——作用中專案為實心主色底＋反白字，其餘為灰底；remote tab SHALL 另以方塊內圖示與其下狀態文字可辨識 ready、restoring、offline、needs-reauth 或 error，SHALL NOT 只用顏色表意。作用中的 restoring／error 專案 SHALL 與 ready 同以實心主色方塊標示選取、不呈 disabled 外觀，狀態由狀態文字的語意色承載。tab 的 hover SHALL 為半透明前景色灰底，SHALL NOT 以主色實心反白。tab 總寬超出 tab 條的可捲範圍時 SHALL 可橫向捲動且 SHALL NOT 顯示捲軸；有 tab 未完整顯示的那一側 SHALL 以邊緣淡出提示可捲動，該側沒有隱藏的 tab 時 SHALL NOT 淡出。面板開啟與作用中專案改變時，tab 條 SHALL 捲動至作用中 tab 完整可見（作用中 tab 本來就完整可見、且不落在淡出區內時不捲動）。

點擊 tab SHALL 原地切換作用中專案，SHALL NOT 喚起主視窗、SHALL NOT 收合面板；切換 SHALL 以分頁的 locator key 為把手（與「選單專案切換」需求同語意），local 與 remote 專案 tab 一視同仁，SHALL NOT 因分頁無本機路徑而靜默無反應。active remote tab 尚無 session且為 restoring／error 時，面板 SHALL 以一張精簡復原卡取代討論與生命週期分區；復原卡 SHALL 顯示 workspace、server、繁中狀態摘要及重新連線或重新登入、在 Speclink 中查看詳情／設定的動作，SHALL NOT 顯示上一個 workspace 的資料。直接重新連線 SHALL 由面板動作回流主視窗 store、面板保持開啟且 SHALL NOT 喚起主視窗；使用者明確選取詳情、設定或重新登入 SHALL 顯示主視窗並取得焦點。active remote session 為 offline 時，面板 SHALL 顯示 stale 狀態列並保留最後成功的變更與討論內容，等待既有 worker 自動收斂。

tab 條右端 SHALL 固定「加入專案」動作項，SHALL NOT 隨 tab 捲動：它與專案 tab 同構：上為虛線框「＋」方塊（與主視窗圖示列的「＋」同形），其下為「加入專案」文字（SHALL NOT 截斷，寬度至少與專案 tab 相同），SHALL 帶語意化 label「加入專案」；點擊 SHALL 先顯示主視窗（含切換至其所在桌面——確保後續對話框於使用者眼前可見）再開啟資料夾選擇器（與主視窗「開啟專案」同語意）——選定即以分頁加入該專案並成為作用中專案，取消則無任何變化。資料夾選擇器等系統原生對話框 SHALL 跟隨系統語言呈現（app SHALL 宣告繁體中文在地化，不得固定英文介面）。

active workspace 為 ready 或已有 session 的 offline 時，面板內容 SHALL 依區塊排列：專案 tab 條卡之下依序為生命週期卡（提案中→進行中→已就緒）、討論卡（「討論」分區常駐呈現）、已轉出卡（有料才現，依「討論列表」需求）、動作區塊卡（單列：左為「開啟 Speclink」文字鈕，右為「專案設定」「設定」「結束」三顆圖示鈕）。面板 SHALL 為三段式版面：專案 tab 條卡為固定頁首、動作區塊卡為固定頁尾，兩者 SHALL 常駐可見、SHALL NOT 隨內容捲動；頁首與頁尾之間的內容卡（生命週期、討論、已轉出，或復原卡／stale 狀態列）同屬可捲中段。區塊之間與卡之間 SHALL 只以間距分隔，SHALL NOT 出現分割線。中段「生命週期在前、討論在後」與原生選單的區段順序（依「系統匣圖示與原生選單」需求）一致；動作區塊卡的單列排法只屬於面板，原生選單動作區的逐項呈現不受本段影響。無 session 的 restoring／error 狀態 SHALL 以「tab 條卡、復原卡、動作區塊卡」排列，不渲染討論與生命週期空卡——復原卡同屬可捲中段，頁首頁尾結構不因此分支。

生命週期分區與討論分區 SHALL 各自以白底細框 12px 圓角卡呈現，分區標題為 12px 粗體、含所屬色相的分區圖示，並 SHALL 顯示該分區的項目計數（徽章樣式與看板欄計數同語彙）；已轉出卡同為白底細框 12px 圓角卡、標題 12px 粗體，圖示與計數徽章 SHALL 為中性灰（依 desktop-app「介面狀態語意色分層」）。標題之下有內容（列、骨架或失敗提示）時 SHALL 有一條細線。卡內的列（變更列、討論列、溢出列）SHALL 以細線分隔、整列 hover 為半透明前景色灰底，SHALL NOT 以主色實心反白；列內的站章、波次數字、複製鈕與進度條在 hover 期間 SHALL 維持本色。生命週期三個階段分區（提案中／進行中／已就緒）SHALL 常駐呈現：零筆階段 SHALL 以「分區標題＋計數 0」的空狀態卡呈現，SHALL NOT 因該階段無變更而整卡消失；全無變更時 SHALL NOT 顯示佔位卡（原「尚無進行中變更」），而以三張計數 0 的分區卡呈現，分區順序固定為提案中→進行中→已就緒。已轉出卡 SHALL 維持有料才現——零筆時 SHALL NOT 呈現（與「討論列表」需求一致）。空狀態卡（討論零筆、生命週期零筆階段）SHALL 維持最小高度、內容垂直置中，不得塌陷成細條。有任務的變更列，其進度條填色 SHALL 依階段套用與看板同源的階段色相（提案中、進行中、已就緒各一色，與看板欄位同色）。

動作區塊卡 SHALL 為單列：左側為「開啟 Speclink」文字鈕（圖示＋文字，佔滿其餘寬度），右側依序為「專案設定」「設定」「結束」三顆圖示鈕（圖示與主視窗側欄同款）；「設定」與「結束」之間 SHALL 有一條細直線分隔符，分隔符兩側的間距 SHALL 大於其餘相鄰按鈕之間的間距，標出「結束」屬另一組、降低誤按（分隔符在卡內，不屬區塊之間的分割線）。三顆圖示鈕 SHALL 各帶與原文字項同名的語意化 label，指標停留時 SHALL 以系統提示顯示該名稱；文字鈕與圖示鈕的 hover SHALL 為半透明前景色灰底。四個動作的行為依「開啟視窗與結束動作」需求，SHALL NOT 因排法改變。

變更與討論列 SHALL 於列尾常駐複製鈕（複製內容與原生選單的複製動作一致：變更為 name、討論為 slug）；複製鈕點擊後 SHALL 短暫顯示成功回饋（勾號圖示，與看板複製鈕同模式）後自行復原。點擊變更或討論列本體 SHALL 顯示主視窗並開啟對應詳情。

面板開啟時 SHALL NOT 有任何互動元素自動取得焦點（不得出現系統焦點框）；複製鈕與動作區塊卡的圖示鈕 SHALL NOT 可經 Tab 鍵取得焦點。新增的復原 tab、card 與動作 SHALL 提供語意化 label 與清楚 pointer hit area。面板高度 SHALL 自適應內容（隨內容增減貼合，達上限高度後 SHALL 僅由可捲中段內部捲動——固定頁首與頁尾 SHALL 全程可見；不得於內容未超限時出現多餘捲動與空白）。中段 SHALL 於捲動時浮現捲動指示、停止捲動後自動隱去，SHALL NOT 常駐顯示、SHALL NOT 完全無指示、SHALL NOT 佔用版面寬度；此浮現與隱去行為 SHALL NOT 依賴系統捲軸偏好——使用者將系統設為「總是顯示捲軸」時中段指示仍 SHALL 自動隱去。中段上下緣 SHALL 依該側是否還有被捲出可視範圍的內容淡出（與 tab 條的邊緣淡出同規則）：已向下捲動時上緣淡出、下方還有內容時下緣淡出，該側沒有被遮住的內容時 SHALL NOT 淡出；淡出只作用於中段，固定頁首與頁尾 SHALL NOT 被淡出。面板失焦時 SHALL 自動收合。面板視窗建立失敗時 app SHALL 以原生選單樣式運作（選單實作跨平台保留、兼作 macOS 與 Windows 的失敗後備；Windows 退回後主鍵點擊圖示亦顯示原生選單）並於設定頁本機設定簽浮出單行錯誤。


平台差異——macOS：點擊 SHALL NOT 顯示原生下拉選單；主鍵（左鍵）與次要鍵（右鍵）點擊圖示 SHALL 完全等價，皆為開閉面板；面板 SHALL 於圖示下方彈出並貼齊圖示。面板開啟 SHALL NOT 奪取目前前景 app 的焦點，SHALL NOT 將面板改為會奪取前景 app 焦點的 key window。面板底 SHALL 於毛玻璃之上、內容之下鋪一層側欄面色（--sidebar token）約六成不透明的補光層（隨淺色／深色主題切換），SHALL NOT 再疊主色漸層。白底卡片 SHALL 維持不透明：卡內文字與列的亮度 SHALL 錨定卡片面色，深色背景下 SHALL NOT 因背後內容而偏暗。毛玻璃 SHALL 自面板外框（卡片與面板邊緣之間的留白）與卡片之間的間距透出、可見背後內容的色調，SHALL NOT 以全不透明底完全遮蔽，亦 SHALL NOT 因補光層過厚而幾乎不可見。

平台差異——Windows：主鍵（左鍵）點擊圖示 SHALL 開閉面板；次要鍵（右鍵）點擊 SHALL 顯示原生下拉選單（內容依「系統匣圖示與原生選單」需求），SHALL NOT 開啟面板（面板開啟中時，點擊圖示會使面板失焦，面板依失焦自動收合規定收合）。面板 SHALL 以圖示為水平中心彈出並貼齊工作列：工作列位於螢幕底部時 SHALL 於圖示上方並使底部貼齊工作列；工作列位於螢幕頂端時 SHALL 改於圖示下方並使頂部貼齊工作列。工作列在哪一側 SHALL 只依圖示所在螢幕判斷（圖示位於該螢幕可用範圍的垂直中線以下即為底部），SHALL NOT 因其他螢幕的相對位置（如副螢幕位於主螢幕上方）而把面板放到另一個螢幕。面板 SHALL 完整落在圖示所在螢幕的可用範圍內：以圖示為中心會超出範圍時 SHALL 水平往內移至貼齊範圍邊緣。面板高度改變（收合、展開、資料增減）後 SHALL 重新定位，使貼齊工作列的那一邊維持貼齊，SHALL NOT 離開工作列或蓋到工作列。面板開啟時 SHALL 取得焦點；面板因失焦收合後 300ms 內收到的開啟要求 SHALL 忽略——以主鍵點擊圖示關閉面板時 SHALL NOT 出現先收合又重新打開的閃現。面板底 SHALL 為不透明側欄面色（--sidebar token），SHALL NOT 鋪半透明補光層或毛玻璃材質，SHALL NOT 自繪圓角；面板視窗 SHALL 無系統標題列與外框並帶系統陰影（Windows 11 由系統呈現圓角）。

#### Scenario: 面板樣式下點擊圖示彈出貼齊面板

- **WHEN** 使用者於 macOS 點擊系統匣圖示
- **THEN** 圖示下方彈出貼齊圖示的面板，頂部為專案 tab 條，其下依作用中 workspace 狀態呈正常資料分區或復原卡，尾端為動作區塊，未出現原生下拉選單

#### Scenario: 右鍵點擊圖示與左鍵等價

- **WHEN** 使用者於 macOS 以滑鼠次要鍵（右鍵）點擊系統匣圖示
- **THEN** 面板於圖示下方彈出且貼齊位置與左鍵點擊一致；面板已開啟時再以右鍵點擊圖示則面板收合——與左鍵行為完全相同

#### Scenario: ready workspace 的區塊順序與卡片

- **WHEN** 面板開啟，作用中 ready 專案存在討論中討論、已轉出討論與各階段變更
- **THEN** 由上而下依序為：專案 tab 條卡、「提案中」「進行中」「已就緒」三張卡、「討論」卡、「已轉出」卡、動作區塊卡（單列：左為「開啟 Speclink」，右為「專案設定」「設定」「結束」三顆圖示鈕）；每張卡為白底細框 12px 圓角，卡與卡之間只有間距、無任何分割線

#### Scenario: 動作區塊卡單列與圖示鈕名稱

- **WHEN** 使用者於 macOS 開啟面板，指標停在動作區塊卡右側最後一顆圖示鈕上，之後點擊右側第一顆圖示鈕
- **THEN** 動作區塊卡只有一列：左為「開啟 Speclink」文字鈕，右為三顆圖示鈕，語意化 label 依序為「專案設定」「設定」「結束」，「設定」與「結束」之間有一條細直線分隔符；指標停留的圖示鈕顯示名稱「結束」；點擊「專案設定」圖示鈕後主視窗顯示並切換至專案設定頁

#### Scenario: 內容超限時頁首頁尾常駐僅中段捲動

- **WHEN** 面板內容高度超過上限，使用者於面板內向下捲動
- **THEN** 專案 tab 條與動作區塊全程固定可見，僅中段內容分區隨捲動移動；捲動指示僅於捲動時浮現、停止後自動隱去，未見常駐捲軸，且系統設為「總是顯示捲軸」時亦然

#### Scenario: 中段上下緣淡出提示還有內容

- **WHEN** 面板內容高度超過上限，使用者把中段從最上方捲到最下方
- **THEN** 仍有內容被捲出可視範圍的一側邊緣淡出，捲到該側盡頭後該側不再淡出；內容未超限時上下皆不淡出；專案 tab 條卡與動作區塊卡全程不被淡出

##### Example: 中段淡出的邊緣

| 捲動位置 | 上緣淡出 | 下緣淡出 |
| -------- | -------- | -------- |
| 最上方 | 否 | 是 |
| 中間 | 是 | 是 |
| 最下方 | 是 | 否 |
| 內容未超限 | 否 | 否 |

#### Scenario: 深色背景下面板亮度不明顯下沉

- **WHEN** 系統為淺色模式，面板於深色背景（如全螢幕深色 IDE）之上開啟
- **THEN** 白底分區卡維持不透明、卡內文字與列的亮度未偏暗；面板外框與卡片之間的縫隙為約六成不透明的側欄面色，透出背後內容的色調但未明顯發黑

#### Scenario: 毛玻璃自外框與卡片縫隙透出

- **WHEN** 系統為淺色模式，面板於色彩鮮明的桌布（例如整片藍色）之上開啟
- **THEN** 面板外框與卡片之間的縫隙帶有桌布的藍色色調，各張卡片本身為不透明白底、看不到桌布

#### Scenario: 點擊專案 tab 原地切換

- **WHEN** 面板開啟且有兩個以上 ready 專案分頁，使用者點擊非作用中專案的 tab
- **THEN** 該 tab 的首字母方塊轉為實心主色（與圖示列作用中方塊同形），面板下方的變更與討論內容切換為該專案，主視窗未被喚起、面板保持開啟

#### Scenario: 點擊 remote 專案 tab 原地切換

- **WHEN** 面板開啟且分頁中含一個非作用中的 ready remote 專案，使用者點擊該 remote 專案的 tab
- **THEN** 該 tab 的方塊轉為實心主色作用中標示，面板下方的變更與討論內容切換為該 remote 專案，主視窗未被喚起、面板保持開啟

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

- **WHEN** 主視窗位於另一個桌面或未在前景，使用者點擊 tab 條右端的「加入專案」項並於資料夾選擇器選定一個專案目錄
- **THEN** 主視窗先被喚起（桌面切換至其所在處）、資料夾選擇器於前景出現；選定後該專案以分頁加入並成為作用中專案；於選擇器按取消則分頁無任何變化

#### Scenario: 分區標題顯示項目計數

- **WHEN** ready workspace 的面板列出提案中 1 筆變更、討論 0 筆
- **THEN** 「提案中」分區標題帶計數徽章 1；討論空狀態卡顯示計數 0 且維持最小高度、內容垂直置中

#### Scenario: 分區卡與列的外觀

- **WHEN** ready workspace 的面板列出一個有三筆以上項目的分區
- **THEN** 該分區為白底細框 12px 圓角卡；標題為 12px 粗體＋色相圖示＋計數徽章、其下一條細線；列之間以細線分隔；指標停在任一列上時整列呈半透明前景色灰底，列內站章、複製鈕與進度條顏色不變，未出現主色實心反白

#### Scenario: 專案 tab 方塊與圖示列同形

- **WHEN** 同一專案同時顯示於主視窗圖示列與面板 tab 條
- **THEN** 兩處的首字母方塊尺寸、圓角與配色相同（作用中實心主色白字、其餘灰底），面板 tab 的專案名列於方塊之下；tab 條右端固定的「加入專案」為虛線框「＋」方塊、其下為「加入專案」文字，方塊與圖示列的「＋」同形

#### Scenario: 專案很多時加入專案固定可見

- **WHEN** 已開啟 6 個專案，tab 條放不下全部 tab
- **THEN** 「加入專案」的「＋」方塊固定在 tab 條右端可見；tab 可橫向捲動，捲動時「＋」位置不變；「＋」方塊下方顯示「加入專案」文字

#### Scenario: tab 條邊緣淡出提示可捲動

- **WHEN** tab 條的 tab 總寬超出可捲範圍，使用者把 tab 條從最左端捲到最右端
- **THEN** 仍有 tab 未完整顯示的那一側邊緣淡出，捲到該側盡頭後該側不再淡出；tab 總寬未超出時兩側皆不淡出；全程不顯示捲軸

##### Example: 淡出的側邊

| 捲動位置 | 左側淡出 | 右側淡出 |
| -------- | -------- | -------- |
| 最左端 | 否 | 是 |
| 中間 | 是 | 是 |
| 最右端 | 是 | 否 |
| tab 總寬未超出 | 否 | 否 |

#### Scenario: 開啟面板時捲到作用中專案

- **WHEN** 已開啟 6 個專案、作用中的是第 6 個，使用者開啟面板
- **THEN** tab 條自動捲動，使第 6 個專案的 tab 完整可見；左側因有隱藏的 tab 而淡出

#### Scenario: 全無變更時三個生命週期分區常駐

- **WHEN** 作用中 ready 專案沒有任何變更，使用者於 macOS 開啟面板
- **THEN** 面板依序呈現「提案中」「進行中」「已就緒」三張分區卡，各帶計數徽章 0、維持最小高度且內容垂直置中，未出現「尚無進行中變更」佔位卡

#### Scenario: 部分有資料時空階段分區仍常駐

- **WHEN** 作用中 ready 專案僅有 1 個進行中變更，無提案中與已就緒變更
- **THEN** 「進行中」分區卡帶計數徽章 1 並列出該變更；「提案中」與「已就緒」分區卡仍呈現且各帶計數徽章 0，三張分區卡依提案中→進行中→已就緒順序排列

#### Scenario: 進度條依階段色相

- **WHEN** ready workspace 的面板同時列出提案中與進行中各一個有任務的變更
- **THEN** 兩列進度條填色分別為提案中與進行中的階段色、彼此不同，與看板欄位的階段配色同源

#### Scenario: 開啟面板無預設焦點

- **WHEN** 使用者點擊系統匣圖示開啟面板
- **THEN** 面板中無任何元素帶系統焦點框（含第一顆複製鈕或復原動作），可點擊動作仍有語意 label 與可見回饋

#### Scenario: 面板不搶焦點且失焦自動收合

- **WHEN** 使用者於 macOS、其他 app 位於前景時點擊系統匣圖示開啟面板，隨後點擊面板外任意處
- **THEN** 面板開啟期間原前景 app 保持焦點；點擊面板外後面板自動收合

#### Scenario: 面板內以常駐複製鈕複製

- **WHEN** 使用者點擊面板中某討論列列尾的複製鈕（無需 hover 顯示、常駐可見）
- **THEN** 系統剪貼簿內容等於該討論的 slug，複製鈕短暫轉為勾號回饋後復原，面板保持開啟、未開啟主視窗

#### Scenario: 面板高度自適應內容

- **WHEN** 面板開啟且正常內容或復原卡高度少於一屏
- **THEN** 面板高度貼合內容（下方無大片空白），內容增加超過上限高度後僅可捲中段出現捲動、頁首頁尾維持固定

#### Scenario: 面板建立失敗退回原生選單

- **WHEN** macOS 上面板視窗建立失敗
- **THEN** 系統匣以原生選單樣式運作，remote error workspace 仍可經復原 submenu 操作，設定頁本機設定簽浮出單行面板建立錯誤

#### Scenario: Windows 主鍵點擊圖示於上方彈出面板

- **WHEN** 使用者於 Windows 11（工作列位於螢幕底部）以主鍵點擊系統匣圖示
- **THEN** 面板出現在圖示正上方、以圖示為水平中心、底部貼齊工作列並取得焦點；內容與版面與 macOS 面板相同（專案 tab 條、提案中→進行中→已就緒→討論、單列動作區塊卡），未出現原生下拉選單

#### Scenario: Windows 次要鍵點擊圖示顯示原生選單

- **WHEN** Windows 面板未開啟時，使用者以次要鍵點擊系統匣圖示
- **THEN** 顯示原生下拉選單（專案區、生命週期分區、討論區、動作區「開啟 Speclink」「專案設定」「設定」「結束」），面板未因此開啟

#### Scenario: Windows 工作列在頂端時面板改於下方

- **WHEN** Windows 工作列位於螢幕頂端，使用者以主鍵點擊系統匣圖示
- **THEN** 面板出現在圖示下方、頂部貼齊工作列

#### Scenario: Windows 副螢幕位於主螢幕上方時面板留在圖示所在螢幕

- **WHEN** Windows 副螢幕位於主螢幕上方、工作列在副螢幕底部，使用者以主鍵點擊副螢幕上的系統匣圖示
- **THEN** 面板出現在圖示上方、底部貼齊副螢幕的工作列，未出現在主螢幕上

##### Example: 垂直位置

| 工作列位置 | 圖示中心 y | 螢幕可用範圍（上緣–下緣） | 面板高 | 面板上緣 y |
| ---------- | ---------- | ------------------------- | ------ | ---------- |
| 底部 | 1056 | 0–1032 | 420 | 612 |
| 頂端 | 24 | 48–1080 | 420 | 48 |
| 副螢幕底部（副螢幕位於主螢幕上方） | -24 | -1080–-48 | 420 | -468 |

#### Scenario: Windows 圖示靠近螢幕邊緣時面板不超出螢幕

- **WHEN** 系統匣圖示的中心距螢幕可用範圍右緣 60px，使用者以主鍵點擊圖示
- **THEN** 寬 320px 的面板整個落在螢幕可用範圍內，右緣與可用範圍右緣對齊，而非以圖示為中心超出螢幕

##### Example: 水平位置限制

| 圖示中心 x | 螢幕可用範圍 | 面板寬 | 面板左緣 x |
| ---------- | ------------ | ------ | ---------- |
| 1000 | 0–1920 | 320 | 840 |
| 1860 | 0–1920 | 320 | 1600 |
| 100 | 0–1920 | 320 | 0 |

#### Scenario: Windows 面板高度改變後仍貼齊工作列

- **WHEN** Windows 面板開啟且底部貼齊工作列，使用者收起「討論」卡使面板變矮
- **THEN** 面板底部仍貼齊工作列、頂端隨之下移，未離開工作列也未蓋到工作列

#### Scenario: Windows 主鍵點擊圖示關閉面板不閃現

- **WHEN** Windows 面板開啟中，使用者以主鍵點擊系統匣圖示
- **THEN** 面板收起並維持收起，未出現先收起又重新打開的閃現；超過 300ms 後再次點擊圖示則正常開啟

##### Example: 失焦後的重開抑制

| 上次因失焦收合距今 | 收到開啟要求 | 結果 |
| ------------------ | ------------ | ---- |
| 無紀錄 | 是 | 開啟 |
| 120ms | 是 | 忽略 |
| 299ms | 是 | 忽略 |
| 300ms | 是 | 開啟 |

#### Scenario: Windows 面板外觀

- **WHEN** 使用者於 Windows 開啟面板
- **THEN** 面板底為不透明側欄面色、無毛玻璃與半透明補光、無自繪圓角，視窗無標題列與外框；Windows 11 上面板四角為系統圓角並帶系統陰影

#### Scenario: Windows 面板失焦自動收合

- **WHEN** Windows 面板開啟中，使用者點擊面板外的任意處（例如其他視窗）
- **THEN** 面板自動收起

#### Scenario: Windows 面板建立失敗退回原生選單

- **WHEN** 於 Windows 面板視窗建立失敗，使用者以主鍵點擊系統匣圖示
- **THEN** 顯示原生下拉選單，設定頁本機設定簽浮出單行面板建立錯誤

#### Scenario: Windows 面板共用收合記憶

- **WHEN** 使用者於 Windows 面板收起「進行中」卡，關閉面板後再以主鍵點擊系統匣圖示開啟
- **THEN** 進行中卡仍只剩標題一行，其餘三張卡維持展開


<!-- @trace
source: tray-promoted-separation
updated: 2026-10-14T23:04:04+08:00
-->
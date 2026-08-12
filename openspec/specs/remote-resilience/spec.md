# remote-resilience Specification

## Purpose

remote 模式失聯與恢復的行為：離線狀態機為單一真相且明確呈現給使用者、離線時最後一份 snapshot 唯讀而寫入即拒（不做離線佇列）、連線恢復後自動收斂並清除 stale 標記，以及重新認證後原地復活。本 capability 保證失聯不會退回 local 模式、不會累積之後才發現衝突的離線寫入，破壞性操作的確認在各處一致。

## Requirements

### Requirement: 離線狀態機單一真相且明確呈現

connection 的 online｜offline｜needs-reauth 狀態 SHALL 由 Rust 端 runtime 單一判定並以事件廣播（connectionId、狀態、訊息）：請求連續失敗達閾值或事件 worker 退避中 sync-state 亦失敗即 offline，任一請求成功或 worker 收斂成功即回 online；needs-reauth SHALL 優先於 offline 呈現。remote 分頁 SHALL 於 offline 與 needs-reauth 各自呈現分頁層級的明確狀態（橫幅與 cloud 狀態圖示）；TS 層 SHALL NOT 自行推斷連線狀態。好天氣路徑（無失敗）SHALL 零改動。

#### Scenario: server 不可達轉為離線呈現

- **WHEN** remote 分頁開啟期間 server 程序被終止，後續請求連續失敗達閾值
- **THEN** 該連線廣播 offline，分頁呈現離線橫幅與 cloud-off 圖示；本地分頁不受影響


<!-- @trace
source: offline-stale-reauth
updated: 2026-07-21
code:
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
-->

---
### Requirement: 最後 snapshot 唯讀且寫入即拒無佇列

offline 或 needs-reauth 期間：已載入的看板與文件內容 SHALL 保留可讀並標示 stale——查詢失敗 SHALL NOT 清空既有內容；全部寫入操作（任務勾選、動詞、artifact 寫回、policy 儲存）SHALL 於 UI 停用（capability 疊加離線遮罩）且 Rust 端命令 SHALL 立即拒絕——SHALL NOT 排隊、暫存或延後重放，恢復後 server 端 SHALL 不存在離線期間的任何寫入。讀取命令 SHALL 放行嘗試（成功即促成回 online）。

#### Scenario: 離線期間看板可讀寫入被拒

- **WHEN** 連線 offline 時使用者檢視看板並嘗試勾選任務
- **THEN** 看板呈現最後成功載入的內容附 stale 標示，勾選操作被停用；即使繞過 UI 呼叫寫入命令亦立即被拒，server 恢復後查無該寫入


<!-- @trace
source: offline-stale-reauth
updated: 2026-07-21
code:
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
-->

---
### Requirement: 恢復自動收斂並清除 stale

server 恢復可達後 SHALL 全自動：事件 worker 以既有 Polling 加 ETag 收斂機制重連，runtime 回 online 並發全量失效通知，store 全量重查後清除 stale 標示——SHALL NOT 要求使用者手動重整或任何操作。同一 connection 的多個 remote session 或 worker 同時需要認證恢復時，Desktop SHALL 讓同一時刻最多一個呼叫消耗已儲存的 refresh credential，其他呼叫 SHALL 共用該次成功換發的 access token 與已輪替 credential，再各自重試原讀取；本機併發 SHALL NOT 被伺服器視為舊 credential 重放，SHALL NOT 因而撤銷 credential family 或進入 `needs-reauth`。伺服器明確拒絕已撤銷、失效或真正遭重放的 credential 時，Desktop SHALL 維持既有 `needs-reauth` 行為。

#### Scenario: server 重啟後自動復原

- **WHEN** offline 期間另一 client 於同 scope 建立新 change，隨後 server 恢復
- **THEN** 分頁自動回 online、stale 標示消失，看板含恢復期間的新 change，全程無使用者操作

#### Scenario: 同來源多分頁併發恢復只輪替一次

- **WHEN** 同一 connection 的兩個 remote 分頁在 server 恢復後同時以失效 access token 發出讀取，且 Keychain 中只有同一枚可用 refresh credential
- **THEN** Desktop 只讓一個 refresh 請求消耗該 credential，兩個分頁共用成功結果後自動回 online，credential family 維持有效且全程不呈現 `needs-reauth`

#### Scenario: 明確撤銷仍進入重新驗證

- **WHEN** server 已明確撤銷該 connection 的 credential family，任一 remote session 嘗試恢復
- **THEN** Desktop 進入 `needs-reauth` 並提供既有重新登入路徑，不持續重試被拒絕的 credential


<!-- @trace
source: fix-multitab-reconnect-recovery
updated: 2026-07-22
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/store.ts
-->

---
### Requirement: 重新認證原地復活不退 local

needs-reauth 時，已建立 session 的 remote 分頁橫幅與尚未建立 session 的 remote 復原頁 SHALL 提供重新登入入口（重用既有 device login／PAT 流程）；macOS Tray Panel 與原生 Tray 選單 SHALL 由同一狀態提供對應重新登入動作。使用者明確選取重新登入 SHALL 顯示主視窗、進入應用程式設定的伺服器頁並聚焦該 connection；僅顯示 needs-reauth 狀態或執行不需登入 UI 的 retry SHALL NOT 自動喚起主視窗。

登入成功後 SHALL 自動對該 connection 的全部 remote sessions 與無 session 復原分頁重走 handshake、全量重查並重啟事件 worker：既有 session 與分頁 SHALL 原地恢復，無 session 分頁 SHALL 於同一 locator key 建立 session。全程分頁 SHALL NOT 消失、SHALL NOT 退回 local mode；既有 session 期間內容維持 stale 唯讀，無 session 分頁 SHALL 維持復原頁且 SHALL NOT 顯示偽造 stale 內容。

#### Scenario: 撤銷 device family 後原地復活

- **WHEN** server 端撤銷該裝置的 device family，使用者於 needs-reauth 橫幅或復原頁選擇重新登入並完成授權
- **THEN** 分頁未曾消失，登入後自動 re-handshake 與重查，看板回到可讀寫狀態，Tray 同步回到 ready

#### Scenario: Tray 顯示 needs-reauth 但不自動奪焦

- **WHEN** background remote workspace 進入 needs-reauth，使用者開啟 macOS Panel 或原生 Tray 選單但尚未選擇重新登入
- **THEN** Tray 顯示需要登入的狀態與動作，主視窗的顯示、Space 與焦點維持不變

#### Scenario: 從 Tray 明確選擇重新登入

- **WHEN** 使用者於 macOS Panel 或原生 Tray 選單的 needs-reauth workspace 選擇重新登入
- **THEN** 主視窗顯示並取得焦點，切至伺服器設定且聚焦對應 connection，登入成功後該 workspace 原地恢復


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
### Requirement: remote 破壞性操作確認一致

remote 分頁的 archive 確認對話 SHALL 沿用與本地相同的確認路徑，描述 SHALL 指出將寫入 server 上的 scope（Project/Repo 名）；deleteChange 於 remote SHALL 維持停用；offline 期間 archive SHALL 隨寫入遮罩停用。

#### Scenario: remote archive 確認指出 scope

- **WHEN** 於 remote 分頁對就緒的 change 觸發 archive
- **THEN** 確認對話呈現且描述含該 Project/Repo 名；確認後寫入 server，取消則無任何變更

<!-- @trace
source: offline-stale-reauth
updated: 2026-07-21
code:
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/views/AppSettingsView.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
-->

---
### Requirement: stale snapshot 與無 session 復原頁依 session 存在性分流

Desktop SHALL 以可用 WorkspaceSession 是否存在裁決 remote 壞天氣呈現：session 已存在而 connection 為 offline 或 needs-reauth 時 SHALL 保留最後成功資料並標示 stale；app 重啟或恢復 handshake 尚未成功、因此無 session 時 SHALL 呈 restoring／error 復原頁，SHALL NOT 讀取上一個 workspace 資料、將空集合標為 stale 或建立離線資料副本。兩條路徑 SHALL 共用 Rust 提供的連線／錯誤真相，TS SHALL NOT 由查詢失敗次數自行推斷 offline。

#### Scenario: 已建立 session 離線保留最後內容

- **WHEN** remote workspace 已成功載入看板後 server 中斷並由 runtime 判定 offline
- **THEN** 主視窗與 macOS Panel 標示 offline／stale 並保留最後成功內容，寫入維持停用，既有 worker 繼續自動收斂

#### Scenario: 重啟後 handshake 失敗不偽造 stale

- **WHEN** app 重啟還原 remote locator，但第一次 handshake 因 server 不可達而失敗且尚無 session
- **THEN** 主視窗與 Tray 呈 error 復原 UI，不呈現上一個 workspace 的變更／討論，也不標示任何空集合為 stale

#### Scenario: server 恢復後兩條路徑各自收斂

- **WHEN** server 恢復可達
- **THEN** 已建立 session 的 offline 路徑由既有 worker 自動回 online；無 session error 路徑於 retry 或登入恢復編排成功後建立 session，兩者均清除對應壞天氣呈現

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

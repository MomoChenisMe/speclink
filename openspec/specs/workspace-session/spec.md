# workspace-session Specification

## Purpose

TBD - created by archiving change 'workspace-session-model'. Update Purpose after archive.

## Requirements

### Requirement: 分頁身分為 WorkspaceLocator 而非 root 路徑

Desktop 分頁 SHALL 以 WorkspaceLocator 為身分：local 變體攜帶 root 路徑，remote 變體（connectionId／projectId／repoId／可選 checkoutRoot）經 chooser 或 remote marker 探測的 handshake 成功路徑建構，checkoutRoot 由 checkout 綁定流程寫入且不參與分頁身分（locator key 不含 checkoutRoot——同 scope 重綁不同 checkout 為同一分頁、新值覆寫舊值）。分頁去重、活躍分頁記錄與 tray 選單識別 SHALL 一律經 locator key（local 為 local:{root}），SHALL NOT 再以裸 root 字串比對。local 分頁的 UI 可觀察行為（分頁列呈現、切換、關閉、上限淘汰、tray 顯示）SHALL 與 root 字串時代一致。

#### Scenario: 同一專案重複開啟仍去重

- **WHEN** 使用者對已在分頁列的資料夾再次執行開啟
- **THEN** 分頁列不新增條目，既有分頁更新顯示名並成為活躍分頁，與重構前行為一致

#### Scenario: 同 scope 重綁 checkout 不分裂分頁

- **WHEN** 對已開啟的 remote 分頁以另一資料夾重新完成 checkout 綁定
- **THEN** 分頁列仍為同一分頁，checkoutRoot 更新為新資料夾


<!-- @trace
source: workspace-chooser-onboarding
updated: 2026-07-20
code:
  - Cargo.lock
  - apps/desktop/core/src/project.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspaceChooser.test.tsx
  - apps/desktop/src/adapter/connections.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/components/WorkspaceChooser.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
-->

---
### Requirement: 分頁持久化 v2 與 v1 靜默遷移

分頁持久化 SHALL 升為 v2 格式（version 欄位、tabs 條目攜帶 locator 與顯示名、activeKey 為 locator key）；讀取到無 version 欄位的舊 v1 格式（root＋name 條目、activeRoot）時 SHALL 靜默遷移——root 逐條映射為 local locator、activeRoot 映射為對應 locator key，下次寫入即為 v2。壞 JSON 或不識別形狀 SHALL 歸零分頁（沿用既有行為）；v1 條目中 root 非字串者 SHALL 丟棄該條目。

#### Scenario: 舊版使用者升級後分頁完整保留

- **WHEN** localStorage 存有 v1 格式（兩個專案分頁與 activeRoot）時啟動新版 app
- **THEN** 分頁列呈現同樣兩個專案、活躍分頁一致，重啟一次後持久化內容為 v2 格式

#### Scenario: 壞 JSON 歸零

- **WHEN** localStorage 的分頁鍵被手改為無法解析的內容後啟動 app
- **THEN** app 以零分頁啟動，不崩潰、不殘留錯誤彈窗


<!-- @trace
source: workspace-session-model
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: 每個 session 自帶 dataSource 且 Rust 側無 current-root 全域

每個 WorkspaceSession SHALL 攜帶自己的 dataSource／settings／events；App SHALL NOT 注入單一全域 DataSource。local session 的 dataSource 與 settings SHALL 將 root 綁入閉包，使每一支 Tauri command 呼叫皆顯式攜帶 root 參數並直通 desktop-core 的帶路徑函式；Rust 側 SHALL NOT 保有 current-root 可變全域，專案探測命令 SHALL 為純探測、對同一路徑重複呼叫冪等且無任何全域副作用。分頁切換後，前一分頁尚未完成的呼叫 SHALL 仍以其原 root 結算，SHALL NOT 落在新分頁的 root 上。

#### Scenario: in-flight 呼叫不受切換影響

- **WHEN** 分頁 A 的清單查詢尚未回應時使用者切到分頁 B
- **THEN** 該查詢仍以 A 的 root 結算（回應內容屬 A），B 的載入以 B 的 root 進行，兩者互不污染

#### Scenario: 設定讀寫落在正確專案

- **WHEN** 分頁列有 A、B 兩專案且活躍為 B，使用者於設定頁修改 Workflow 欄位
- **THEN** 寫入落在 B 的 openspec/config.yaml，A 的設定檔內容不變


<!-- @trace
source: workspace-session-model
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: watcher 顯式跟隨活躍 session 且事件攜帶 root

檔案監看 SHALL 維持單一實例並跟隨活躍 session：重掛 SHALL 由顯式的監看命令觸發（探測命令 SHALL NOT 附帶重掛副作用）；workspace-changed 事件 SHALL 攜帶被監看的 root，session 的事件來源 SHALL 以自身 locator 過濾後才觸發重載。監看不可用時僅失去自動刷新、app 照常運作的既有語意 SHALL 保留。外部寫者修改活躍專案 openspec/ 後看板秒級自動更新的既有行為 SHALL 不變。

#### Scenario: 外部變更僅觸發活躍 session 重載

- **WHEN** 活躍分頁為 A 時，外部寫者修改 A 的 openspec/ 下文件
- **THEN** A 的看板數秒內自動更新；事件 payload 為 A 的 root，非活躍分頁不因此發出任何查詢


<!-- @trace
source: workspace-session-model
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: 重構行為凍結

本次重構 SHALL 維持 UI 可觀察行為與重構前一致：既有桌面測試套件（分頁、store、workspace、App、tray、tray panel、設定頁）SHALL 全數通過且斷言語意不弱化；packages/ui SHALL 零改動；桌面 app SHALL 於真實視窗完成手動驗證——分頁互切、設定讀寫、外部變更即時反映、v1 持久化遷移、tray 切換與重啟恢復。

#### Scenario: 既有測試與建置全綠

- **WHEN** 重構完成後執行桌面測試與建置（npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop）
- **THEN** 全數通過，packages/ui 無任何檔案差異

<!-- @trace
source: workspace-session-model
updated: 2026-07-17
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/session.test.ts
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/__tests__/tray.test.ts
  - apps/desktop/src/__tests__/trayPanel.test.tsx
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/main.tsx
  - apps/desktop/src/panel/TrayPanel.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/tray.ts
  - apps/desktop/src/views/SettingsView.tsx
-->

---
### Requirement: 可選取的 remote 復原分頁與 session 邊界

已存在於分頁列的 remote workspace 在尚無可用 WorkspaceSession 時，Desktop SHALL 允許該分頁成為作用中 navigation destination，並以 locator key 記錄 activeKey；handshake 進行中 SHALL 呈 restoring，失敗 SHALL 呈 error 復原頁。此狀態下 workspace 資料操作 SHALL 視為無 active session 而不執行，主內容 SHALL NOT 顯示上一個分頁的資料或偽造 stale snapshot。restoring／error 為不持久化的執行期狀態；關閉分頁 SHALL 同時清除，retry 成功 SHALL 於同一 locator key 建立 session 並清除而不新增分頁。

#### Scenario: handshake 失敗仍選取該分頁

- **WHEN** local 分頁作用中，使用者點擊一個持久化但尚無 session 的 remote 分頁，而 handshake 失敗
- **THEN** remote 分頁成為作用中且顯示 error 復原頁，local 分頁資料不再出現在主內容，remote 分頁未消失

#### Scenario: retry 成功原地建立 session

- **WHEN** 作用中的 remote error 分頁再次執行 retry 且 handshake 成功
- **THEN** 同一分頁原地取得 session 並顯示 server 資料，restoring／error 清除，分頁列不新增重複項目

#### Scenario: 較舊 handshake 不搶回作用中分頁

- **WHEN** remote 分頁 A 的 handshake 尚未完成時使用者切至分頁 B，之後 A 的 handshake 才成功或失敗
- **THEN** A 的結果只更新 A 的 session 或 recovery 狀態，activeKey 維持 B

#### Scenario: 同分頁只接受最新 retry 結果

- **WHEN** 同一 remote error 分頁先後觸發兩次 retry，第二次成功後第一個較舊請求才失敗
- **THEN** 該分頁維持第二次成功建立的 session，較舊失敗 SHALL NOT 覆蓋成 error

#### Scenario: local 分頁切換維持既有行為

- **WHEN** 使用者在兩個有效 local 分頁之間切換
- **THEN** active session、watcher、看板資料與持久化 activeKey 依既有流程切換，不建立 remote recovery 狀態

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
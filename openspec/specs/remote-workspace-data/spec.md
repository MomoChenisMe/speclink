# remote-workspace-data Specification

## Purpose

TBD - created by archiving change 'remote-data-source'. Update Purpose after archive.

## Requirements

### Requirement: handshake 成功後才建立 remote session

remote session SHALL 僅在 binding handshake 成功後建立：新開啟入口（chooser 的 scopes 清單選擇或 remote marker 探測）以選定的 repo 發起 handshake，成功時以回應中的 project/repo 識別建構 remote locator 與分頁；失敗（未授權、不存在、多義）SHALL 原樣提供 technical detail 且 SHALL NOT 建立分頁或 session。scopes 清單雖經 membership 過濾，選擇與 handshake 之間權限可能變化，handshake 仍為最終防線。

重啟後恢復或選取既有 remote 分頁 SHALL 重走 handshake：分頁須先成為作用中並呈 restoring；成功 SHALL 於同一 locator key 原地建立 session，失敗 SHALL 保留作用中分頁並呈現 error 復原頁與 retry／設定或重新登入動作，SHALL NOT 靜默消失、退回本地模式或顯示上一 workspace 資料。retry SHALL 重走相同 handshake 前置，成功前 SHALL NOT 建立 session。

#### Scenario: 新開啟入口 handshake 失敗不建分頁

- **WHEN** 於 scopes 清單選定 repo 後、handshake 前該使用者的 membership 被撤銷
- **THEN** handshake 被拒並於開啟入口呈現錯誤，分頁列不出現新分頁，session 清單不新增項目

#### Scenario: 重啟後 remote 分頁恢復成功

- **WHEN** 含 remote 分頁的 app 重啟且 credential 與 scope 仍有效
- **THEN** 該分頁先呈 restoring，handshake 成功後於原位恢復 server 資料，分頁列不新增重複項目

#### Scenario: 重啟後 credential 失效進入復原頁

- **WHEN** 含 remote 分頁的 app 重啟，而該 connection 的 credential 已失效
- **THEN** 該分頁保持存在且成為作用中，呈現需要重新認證的復原頁與對應動作，不顯示上一 workspace 資料

#### Scenario: server 不可達時 retry 原地恢復

- **WHEN** 重啟恢復因 server 不可達而進入 error 復原頁，server 恢復後使用者選擇重新連線
- **THEN** 同一分頁呈 restoring 並重走 handshake，成功後原地建立 session、清除 error 且顯示 server 資料


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
### Requirement: Query 加 ETag 為重讀正典且 push 只做 invalidate

remote session 的資料讀取 SHALL 一律走 Query（清單、文件、狀態）；SSE 事件 SHALL 只作為失效提示觸發重讀，SHALL NOT 攜帶被消費的資料實體。同一 server 的多個 session SHALL 共用單一 SSE 訂閱，失效提示 SHALL 以 locator 對應分發。

#### Scenario: server 側變更經 invalidate 反映

- **WHEN** remote 分頁開啟期間，另一 client 於同 repo 建立新 change
- **THEN** 桌面收到失效提示後重新查詢，看板數秒內出現該 change


<!-- @trace
source: remote-data-source
updated: 2026-07-19
code:
  - Cargo.lock
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/events.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/events_sse.rs
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: 斷線以 Polling 加 ETag 收斂後續訂

SSE 中斷時 Desktop SHALL：停流、以 /sync-state 的 ETag 比對偵測錯過的變更（不同即觸發重載）、以退避重連並帶 Last-Event-ID 續傳；server 回 reset 信號時 SHALL 觸發全量重載後自新事件位點續訂。SSE 持續不可用期間 SHALL 以輪詢維持收斂；恢復後 SHALL 回到事件驅動。全程 SHALL NOT 產生資料遺漏——完全漏掉 push 事件後仍能經 Query 收斂到 server 現況。

#### Scenario: server 重啟後自動收斂

- **WHEN** remote 分頁開啟期間 server 程序重啟，期間該 repo 發生過變更
- **THEN** Desktop 於重連後（經 ETag 比對或 reset 信號）重載至 server 現況，錯過的變更全部反映，無需使用者操作


<!-- @trace
source: remote-data-source
updated: 2026-07-19
code:
  - Cargo.lock
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/events.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/events_sse.rs
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: capability 驅動停用且不偽造缺口

RemoteDataSource SHALL 附帶逐操作的 capability 描述（來源＝handshake 回應與端點覆蓋矩陣）；封存瀏覽、全文搜尋、正典 spec 內文、validate/analyze 動詞、刪除變更與任務拖排 SHALL 直達 server 對應端點、與本地 session 同形呈現（刪除與任務拖排依 role：reader 呈現停用附繁體中文說明）；server 仍無對應端點的操作（看板拖排）SHALL 於 UI 停用並附繁體中文說明，對應 DataSource 方法 SHALL 回拒絕錯誤；SHALL NOT 於 client 端偽造或近似實作缺口。本地 session 的全部操作 SHALL 維持可用且行為零改動。批次任務操作以逐任務寫回組合時，中途失敗 SHALL 中止並回報已完成筆數。

#### Scenario: 動詞與寫入面直達而看板拖排停用

- **WHEN** 以 editor 身分於 remote 分頁執行 validate、analyze、刪除一個變更並拖排一個任務
- **THEN** 四者皆如本地生效並呈現真實結果（刪除後卡片消失、任務落位並重編號）；同時看板卡片拖排維持停用附繁中說明，本地分頁全功能照常

#### Scenario: reader 的寫入面呈現停用

- **WHEN** 以 reader 身分開啟 remote 分頁
- **THEN** 刪除變更與任務拖排呈現停用附繁中說明、validate/analyze 照常可用，對應停用方法回拒絕錯誤


<!-- @trace
source: remote-verb-parity
updated: 2026-07-23
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_verb_parity.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/convert.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/events.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/verb_api.rs
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: token 換發全程 Rust 側且 401 語意固定

remote 請求的 access token SHALL 僅存於 Rust 記憶體；請求遇 401 SHALL 以 Keychain 的 refresh credential 換發一次並重試一次，rotation 後的新 refresh credential SHALL 立即回寫 Keychain；再失敗 SHALL 令該連線進入需重新認證狀態——TS 層 SHALL 只見狀態布林與訊息，SHALL NOT 接觸任何 token。SSE 訂閱的 401 SHALL 同語意。

#### Scenario: access token 過期自動換發

- **WHEN** access token 過期後使用者於 remote 分頁觸發查詢
- **THEN** 查詢經自動 refresh 後成功回應，使用者無感；Keychain 內為 rotation 後的新 refresh credential

#### Scenario: refresh 亦失效即需重新認證

- **WHEN** refresh credential 已被撤銷時觸發查詢
- **THEN** 連線進入需重新認證狀態，操作回拒絕錯誤與繁中訊息，app 不崩潰、本地分頁不受影響

<!-- @trace
source: remote-data-source
updated: 2026-07-19
code:
  - Cargo.lock
  - apps/desktop/src-tauri/src/event_manager.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteOpen.test.ts
  - apps/desktop/src/__tests__/serversPanel.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/remoteDataSource.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/components/ServersPanel.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/session.ts
  - apps/desktop/src/store.ts
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/device.rs
  - crates/speclink-remote/src/events.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/events_sse.rs
  - packages/ui/src/__tests__/boardSearchBar.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/BoardSearchBar.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: remote_open 失敗保留 machine-readable reason

Desktop 的 remote_open 邊界失敗時 SHALL 提供 camelCase 的 message、reason、status 欄位：message 為 technical detail 字串，reason 為 protocol reason 字串或 null，status 為 HTTP status 整數或 null。Desktop SHALL 依 reason／status 與 Rust runtime 狀態正規化為 unreachable、needs-reauth、access-denied、not-found、unknown 五種封閉復原分類；UI 摘要 SHALL 由繁體中文 i18n 文案產生，SHALL NOT 以英文 message 比對分類。失敗 payload SHALL NOT 含 access token、refresh credential、PAT、authorization header 或 Keychain 內容；server HTTP API 與成功 payload SHALL 維持不變。

#### Scenario: transport failure 分類為 unreachable

- **WHEN** remote_open 在取得 HTTP response 前因 server 不可達而失敗
- **THEN** failure status 為 null 且 Desktop 呈 unreachable 摘要、重新連線與伺服器設定動作，technical detail 可由使用者展開

#### Scenario: HTTP status 對應復原分類

- **WHEN** remote_open 分別回傳 401、403、404
- **THEN** Desktop 分別呈 needs-reauth、access-denied、not-found 復原分類，不以 message 文字判斷

##### Example: status 對應

| status | recovery kind | 主要復原方向 |
| ------ | ------------- | ------------ |
| 401 | needs-reauth | 重新登入 |
| 403 | access-denied | 檢查帳號或伺服器設定 |
| 404 | not-found | 檢查 workspace 或移除分頁 |
| null transport | unreachable | 重新連線 |

#### Scenario: 無法解析的 rejection 安全降階

- **WHEN** 測試 adapter 或舊版邊界拒絕一個不符合 structured payload 的字串或未知物件
- **THEN** Desktop 呈 unknown 復原分類並保留可展開 technical detail，app 不崩潰且仍可 retry

#### Scenario: 失敗 payload 不洩漏 credential

- **WHEN** remote_open 因認證失敗回傳 structured rejection
- **THEN** payload 只含 message、reason、status，不含 token、PAT、refresh credential、authorization header 或 Keychain 值

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
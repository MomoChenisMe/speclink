# desktop-config Specification

## Purpose

TBD - created by archiving change 'desktop-config-multiproject'. Update Purpose after archive.

## Requirements

### Requirement: 執行期切換專案 root

桌面 app SHALL 提供「開啟專案」操作：經原生資料夾選擇器選定目錄後，app SHALL 以該目錄為起點向上探索 speclink 專案根（與啟動時的探索語意一致），命中即於執行期切換專案 root——看板、已封存與設定頁 SHALL 全數改為呈現新專案內容，SHALL NOT 要求重啟；頂欄分頁列 SHALL 以 active 分頁標示目前專案（root 目錄名）。開啟失敗（路徑不存在或不可讀）時 app SHALL 顯示單行錯誤訊息並維持原專案不變。

#### Scenario: 選定既有專案後全視圖切換

- **WHEN** 使用者於專案 A 中經「開啟專案」選定含 openspec/ 的專案 B 根目錄
- **THEN** 看板顯示專案 B 的 change、分頁列新增專案 B 的分頁並標示為 active，過程中 app 未重啟，且專案 A 與 B 的檔案內容均未被此操作改動

#### Scenario: 自子目錄向上探索至專案根

- **WHEN** 使用者選定專案 B 根目錄下的子目錄（子目錄本身不含 openspec/）
- **THEN** app 切換至向上探索命中的專案 B 根目錄，行為與直接選定專案根一致

#### Scenario: 開啟失敗維持原專案

- **WHEN** 使用者嘗試開啟一個已被刪除或不可讀的路徑
- **THEN** app 顯示單行錯誤訊息，目前專案與畫面內容維持不變


<!-- @trace
source: desktop-config-multiproject
updated: 2026-07-07
code:
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/locale.test.ts
  - apps/desktop/src/__tests__/messages.test.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/i18n/runtime.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/views/SettingsView.tsx
  - crates/speclink-core/src/config.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/changeListItem.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/i18n.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/checkbox.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/tooltip.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: 未初始化目錄經確認後自動初始化

所選目錄向上探索未命中任何 speclink 專案時，app SHALL NOT 逕行寫入，而 SHALL 顯示初始化確認對話框（含 AI 工具多選 claude／codex，預設勾選 claude）。使用者確認後 app SHALL 執行與 speclink init 等效的初始化（openspec/ 骨架含 specs/、changes/archive/ 與 config.yaml、專案根的 .speclink.yaml 記錄所選 tools、為每個所選工具生成指令檔 marker 區塊與 skills 檔），隨即切換至該專案；使用者取消時 app SHALL 維持原專案，且目標目錄 SHALL NOT 產生任何寫入。初始化失敗時 app SHALL 顯示單行錯誤訊息且 SHALL NOT 切換 root。

#### Scenario: 確認後初始化並切入新專案

- **WHEN** 使用者選定不含任何 speclink 標記的空目錄，於確認對話框保持預設（claude）並確認
- **THEN** 該目錄產生 openspec/（含 specs/、changes/archive/、config.yaml）、.speclink.yaml（tools 含 claude）、CLAUDE.md 的 SPECLINK marker 區塊與 .claude/skills/ 技能檔，且 app 切換至該專案並於看板顯示空清單

#### Scenario: 勾選 codex 時生成對應工具檔

- **WHEN** 使用者於確認對話框加勾 codex 後確認
- **THEN** 目標目錄除 claude 對應檔案外，另產生 AGENTS.md 的 SPECLINK marker 區塊與 .agents/skills/ 技能檔，.speclink.yaml 的 tools 同時記錄 claude 與 codex

#### Scenario: 取消初始化則零寫入

- **WHEN** 使用者於確認對話框取消
- **THEN** app 維持原專案，所選目錄內容與選擇前完全相同（無任何新檔案或目錄）


<!-- @trace
source: desktop-config-multiproject
updated: 2026-07-07
code:
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/locale.test.ts
  - apps/desktop/src/__tests__/messages.test.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/i18n/runtime.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/views/SettingsView.tsx
  - crates/speclink-core/src/config.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/changeListItem.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/i18n.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/checkbox.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/tooltip.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: 專案分頁列存於 app 本機

app 頂欄 SHALL 以分頁列呈現開啟過的專案（路徑與顯示名，上限 10 個分頁）：分頁 SHALL 跨啟動持久化於 app 本機狀態（含順序與最後活躍分頁），SHALL NOT 寫入任何專案目錄。點擊分頁 SHALL 以該路徑執行與「開啟專案」相同的切換語意；同一專案再次開啟 SHALL 去重並移至既有分頁；關閉分頁 SHALL 將其自持久化清單移除。分頁 SHALL 顯示該專案「待收尾數」徽章——已就緒（任務全數完成、等待封存）變更數＋已結論未轉出討論數，已轉出（promoted）討論 SHALL NOT 計入；活躍分頁隨看板刷新即時更新，背景分頁 SHALL 於 app 啟動時各查詢一次、之後保留最後已知值；hover 徽章的說明文字 SHALL 使用待收尾語意。分頁指向已不存在的路徑時 SHALL 以錯誤態呈現，點擊 SHALL 顯示錯誤並提供自分頁移除，SHALL NOT 切換專案。無任何分頁時 app SHALL 顯示「開啟專案」空狀態引導頁而非空看板。app SHALL 支援 Ctrl+Tab 循環切換與 Ctrl+1..9 直達第 N 個分頁。

#### Scenario: 成功開啟後記入分頁並去重上移

- **WHEN** 使用者依序開啟專案 A、B，再次開啟專案 A
- **THEN** 分頁列僅含 A、B 各一個分頁且 A 為 active，A 與 B 的專案目錄內均無因分頁列而新增的檔案；重啟 app 後分頁列還原為相同內容

#### Scenario: 點擊分頁切換專案

- **WHEN** 使用者於專案 A 為 active 時點擊專案 B 的分頁（或按 Ctrl+Tab 循環至 B）
- **THEN** 看板、已封存與設定頁改為呈現專案 B 內容，B 的分頁轉為 active，行為與經「開啟專案」選定 B 一致

#### Scenario: 分頁徽章顯示待收尾數

- **WHEN** 分頁列含專案 A（active，2 個已就緒變更與 1 份已結論未轉出討論）與背景專案 B（啟動時查得待收尾數 1）
- **THEN** A 的分頁徽章顯示 3 並隨看板刷新即時更新（全部封存與轉出後歸零），B 的分頁徽章顯示 1（最後已知值）；hover 徽章顯示待收尾語意的說明文字

##### Example: 待收尾數計算

| 看板狀態 | 計入待收尾 | 說明 |
| -------- | ---------- | ---- |
| 變更 21/21 任務完成（已就緒欄） | 是 | 等待使用者封存 |
| 變更 5/21 進行中 | 否 | agent 工作中，無需使用者動作 |
| 討論 concluded 未轉出 | 是 | 等待轉為變更或封存 |
| 討論 promoted（已轉出） | 否 | 已轉出，隨子變更生命週期收尾 |
| 討論 open（討論中） | 否 | 討論仍在推進 |

#### Scenario: 分頁路徑已消失時轉錯誤態

- **WHEN** 分頁列中專案 B 的目錄已被刪除，使用者點擊該分頁
- **THEN** 該分頁呈現錯誤態（警示標記），app 顯示單行錯誤訊息、維持原專案，並提供「自分頁移除」操作；執行後該分頁自分頁列與持久化清單消失

#### Scenario: 零分頁時顯示空狀態引導頁

- **WHEN** app 於無任何持久化分頁的狀態下啟動（如首次使用）
- **THEN** 主畫面顯示含「開啟專案」操作的空狀態引導頁（說明可選既有專案目錄或經確認初始化一般目錄），而非空白看板


<!-- @trace
source: spec-archive-drawer
updated: 2026-07-11
code:
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - packages/ui/src/__tests__/archivedDrawer.test.tsx
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/specDrawer.test.tsx
  - packages/ui/src/__tests__/specList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/SpecDrawer.tsx
  - packages/ui/src/components/SpecList.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 設定頁圖形化讀寫兩層設定

設定頁 SHALL 以三頁簽組織，標籤依序為 config.yaml、.speclink.yaml、本機設定，預設 SHALL 落在 config.yaml 簽：

- **config.yaml** 簽 SHALL 含「專案說明」卡與「產出規則」卡（行為見需求「設定頁編輯專案說明與產出規則」），及「產出政策」卡——locale、spec_locale（下拉）與 tdd、audit（開關）。
- **.speclink.yaml** 簽 SHALL 含「AI 工具」卡——內建工具 claude／codex 多選，自訂工具描述子原樣呈現為不可編輯項。
- **本機設定** 簽 SHALL 含「介面語言」卡（行為見需求「UI 介面語言支援 zh-TW 與 en」）。

config.yaml 與 .speclink.yaml 簽首 SHALL 以等寬字註記對應檔案路徑；本機設定簽 SHALL 註記其內容僅存於此裝置、不寫入版本庫。讀取時未設定的欄位 SHALL 呈現為預設值狀態；寫入時 SHALL 僅代換目標鍵——未觸及的鍵（remote、spec_dir、自訂工具描述子等）SHALL 原樣保留；政策欄位設回預設值時 SHALL 移除該鍵而非寫入明值。tools 寫入成功後 app SHALL 同步技能檔（新選工具生成、取消工具清理殘留）。自訂工具描述子 SHALL 寫入後保留。任一層設定檔解析失敗時，對應頁簽（config.yaml 簽掛工作流層、.speclink.yaml 簽掛應用層）的標籤 SHALL 帶警示點、簽內 SHALL 浮出解析失敗說明且該簽表單 SHALL 停用；本機設定簽 SHALL NOT 受任何解析失敗影響。

#### Scenario: 三頁簽組織與預設簽

- **WHEN** 使用者開啟設定頁
- **THEN** 頁簽依序為 config.yaml、.speclink.yaml、本機設定且預設落在 config.yaml 簽（含專案說明、產出規則、產出政策三卡，簽首等寬字註記檔案路徑）；切至 .speclink.yaml 簽見 AI 工具卡；切至本機設定簽見介面語言卡與「僅存於此裝置」註記

#### Scenario: 寫入政策欄位且未觸及鍵原樣保留

- **WHEN** config.yaml 原含 rules 區塊與 context 文字，使用者於設定頁將 tdd 切為開啟並儲存
- **THEN** 重新讀取 config.yaml 可見 tdd: true，且 rules 與 context 內容與寫入前逐字元一致

#### Scenario: 設回預設值即移除鍵

- **WHEN** config.yaml 原含 locale: tw，使用者於設定頁將 locale 改回「未設定（English）」並儲存
- **THEN** 重新讀取 config.yaml 已無 locale 鍵，且引擎解析該檔的有效 locale 為預設 English

##### Example: 政策欄位寫入效果

| 操作前檔案狀態 | 表單操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| 無 tdd 鍵 | tdd 切開啟 | 新增 tdd: true |
| tdd: true | tdd 切關閉 | tdd 鍵被移除（預設即 false） |
| locale: tw、含 rules 區塊 | spec_locale 選 auto | 新增 spec_locale: auto，locale 與 rules 原樣保留 |

#### Scenario: tools 變更後技能同步

- **WHEN** .speclink.yaml 原 tools 僅 claude，使用者加選 codex 並儲存
- **THEN** .speclink.yaml 的 tools 記錄 claude 與 codex，且專案根新增 AGENTS.md marker 區塊與 .agents/skills/ 技能檔

#### Scenario: 自訂工具描述子原樣保留

- **WHEN** .speclink.yaml 的 tools 含一個自訂描述子物件，使用者於設定頁變更內建工具勾選並儲存
- **THEN** 寫入後的 tools 清單仍含該描述子且欄位內容不變，設定頁將其呈現為不可編輯項

#### Scenario: 解析失敗簽級警示

- **WHEN** config.yaml 被外部改壞為無法解析，使用者開啟設定頁並停留在本機設定簽
- **THEN** config.yaml 頁簽標籤帶警示點；切至該簽可見解析失敗說明，產出政策卡表單與專案說明、產出規則兩卡的編輯鈕停用；本機設定簽的介面語言三選仍可正常使用

---
### Requirement: 設定寫入具解析驗證且失敗浮出

設定頁載入 SHALL 區分「檔案缺席或欄位未設定」與「檔案存在但解析失敗」：後者 SHALL 於設定頁顯示警告並停用該檔對應表單，app SHALL NOT 對解析失敗的檔案執行寫入。寫入流程 SHALL 於寫檔前驗證新內容可被對應設定解析器解析且目標欄位值正確，寫檔後 SHALL 回讀再次驗證；任一驗證失敗 SHALL 顯示指明檔案與階段的單行錯誤訊息，且磁碟上的檔案 SHALL 維持原內容——SHALL NOT 留下不可解析的設定檔。

#### Scenario: 解析失敗的檔案拒絕寫入

- **WHEN** 使用者手動將 config.yaml 改壞（YAML 語法錯誤）後開啟設定頁
- **THEN** 設定頁對該檔顯示解析失敗警告、對應表單停用，且儲存操作不可對該檔執行

#### Scenario: 寫入驗證失敗檔案不變

- **WHEN** 設定寫入流程於寫檔前驗證未通過
- **THEN** app 顯示指明檔案與失敗階段的單行錯誤訊息，磁碟上該檔內容與操作前逐字元一致


<!-- @trace
source: desktop-config-multiproject
updated: 2026-07-07
code:
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/locale.test.ts
  - apps/desktop/src/__tests__/messages.test.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/i18n/runtime.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/views/SettingsView.tsx
  - crates/speclink-core/src/config.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/changeListItem.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/i18n.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/checkbox.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/tooltip.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: UI 介面語言支援 zh-TW 與 en

app 的 UI 介面語言 SHALL 支援 zh-TW 與 en：未設定偏好時 SHALL 跟隨系統語言（系統語言以 zh 開頭判為 zh-TW，其餘判為 en）；設定頁 SHALL 提供「跟隨系統／zh-TW／en」三選，切換 SHALL 即時對全介面生效並持久化於 app 本機。UI 語言偏好與 config.yaml 的 locale（AI artifacts 產出語言）SHALL 互不影響。兩語言字典的 key 集合 SHALL 相等；查無 key 時 SHALL 顯示 key 本身而非另一語言的字串。

#### Scenario: 未設定偏好時跟隨系統語言

- **WHEN** app 於 UI 語言偏好未設定的狀態下啟動
- **THEN** 系統語言以 zh 開頭時全介面呈現 zh-TW，否則呈現 en

##### Example: 系統語言判定

| 系統語言 | UI 語言 |
| -------- | ------- |
| zh-TW | zh-TW |
| zh-CN | zh-TW |
| en-US | en |
| ja-JP | en |

#### Scenario: 手動切換即時生效並持久化

- **WHEN** 使用者於設定頁將 UI 語言由 zh-TW 切為 en
- **THEN** 全介面（頂欄、側欄、看板、對話框）即時改為英文，重啟 app 後仍為英文，且 config.yaml 內容未被此操作改動

#### Scenario: UI 語言與 artifacts 產出語言互不影響

- **WHEN** config.yaml 設定 locale: tw，使用者將 UI 語言切為 en
- **THEN** UI 呈現英文，而 config.yaml 的 locale 仍為 tw（引擎產出 artifacts 的語言政策不受 UI 語言影響）

<!-- @trace
source: desktop-config-multiproject
updated: 2026-07-07
code:
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/project.rs
  - apps/desktop/core/src/settings.rs
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/locale.test.ts
  - apps/desktop/src/__tests__/messages.test.ts
  - apps/desktop/src/__tests__/projectTabs.test.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/__tests__/tabs.test.ts
  - apps/desktop/src/__tests__/workspace.test.ts
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/i18n/locale.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/i18n/runtime.ts
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/src/tabs.ts
  - apps/desktop/src/views/SettingsView.tsx
  - crates/speclink-core/src/config.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/changeListItem.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/i18n.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/checkbox.tsx
  - packages/ui/src/components/ui/select.tsx
  - packages/ui/src/components/ui/tooltip.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: 設定頁編輯專案說明與產出規則

設定頁 config.yaml 頁簽 SHALL 呈現「專案說明」與「產出規則」兩張獨立卡，各卡預設唯讀、各持編輯態——一卡進入編輯 SHALL NOT 影響另一卡的唯讀呈現與可用性。專案說明唯讀 SHALL 以 markdown 渲染呈現 context 內容，超過固定高度 SHALL 收合並提供顯示更多展開；未設定時 SHALL 顯示空狀態提示。產出規則唯讀 SHALL 僅列出有條目的 artifact 鍵，鍵名為小節標題、條目為清單。各卡右上編輯鈕 SHALL 就地切換該卡為編輯態並將按鈕列改為取消與儲存：專案說明為 raw markdown 多行文字區；產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵各渲染一個多行文字區，SHALL NOT 提供自由鍵輸入；一行即一條規則，行序 SHALL 即為寫入檔案的條目順序（亦即指令注入順序）。儲存時 SHALL 逐行去除頭尾空白，空行 SHALL NOT 寫入；專案說明清空儲存 SHALL 移除 context 鍵，某鍵文字區清空 SHALL 移除該 artifact 鍵，全部鍵皆空 SHALL 移除 rules 鍵。專案說明卡儲存 SHALL 僅寫入 context、產出規則卡儲存 SHALL 僅寫入 rules——各卡儲存 SHALL NOT 改動另一卡對應的鍵。取消 SHALL 還原該卡唯讀呈現且 SHALL NOT 寫入檔案。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程；序列化 SHALL 為以 YAML 保留字元（反引號、at 符號等）開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時，兩卡 SHALL 依所在頁簽的解析失敗呈現停用編輯（見需求「設定頁圖形化讀寫兩層設定」）。

#### Scenario: 唯讀優先與各卡就地編輯

- **WHEN** 使用者開啟設定頁的 config.yaml 簽
- **THEN** 專案說明卡以 markdown 渲染唯讀（超長時收合並可顯示更多）、產出規則卡僅列有條目的鍵；點專案說明卡的編輯後僅該卡就地變為可編輯（按鈕列為取消與儲存），產出規則卡維持唯讀且仍可獨立進入自身的編輯態

#### Scenario: 各卡儲存僅寫對應鍵

- **WHEN** config.yaml 原含 context 與 rules，使用者僅於產出規則卡進入編輯、修改後儲存
- **THEN** 重新讀取 config.yaml 的 rules 依編輯更新，而 context 值與寫入前逐字元一致

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者於專案說明卡進入編輯，於文字區輸入多行 markdown 並儲存
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致、其餘鍵原樣保留，該卡回唯讀並渲染新內容，重開設定頁呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於產出規則卡某鍵文字區新增一行以 YAML 保留字元開頭的規則並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析（必要引號由寫入自動加上）、該條目值逐字元還原，整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 文字區新增一行「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析，rules 的 tasks 節含值「@完成後執行全部測試」（逐字元一致），proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者於專案說明卡清空文字區全部行並儲存，再於產出規則卡清空某鍵文字區並儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵；其餘鍵原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 編輯操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩行 | 專案說明卡清空並儲存 | context 鍵被移除，rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 產出規則卡清空 tasks 文字區並儲存 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 產出規則卡清空該文字區並儲存 | rules 鍵整個被移除 |

#### Scenario: 行序即寫入順序

- **WHEN** 使用者於產出規則卡某鍵文字區將第二行搬至第一行並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調，後續該 artifact 的指令注入依新順序呈現規則

##### Example: 行對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 於 tasks 文字區將「更新文件」一行搬到「先寫失敗測試」之前並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案進入產出規則卡編輯
- **THEN** 文字區恰為 proposal、design、specs、tasks 四個固定鍵各一，介面不提供自由新增分節鍵的輸入；回唯讀後僅有條目的鍵被列出

#### Scenario: 取消放棄編輯

- **WHEN** 使用者於專案說明卡進入編輯、修改內容後點取消
- **THEN** 該卡還原為編輯前的唯讀呈現，config.yaml 逐字元未變，且產出規則卡全程不受影響

#### Scenario: 解析失敗停用編輯

- **WHEN** config.yaml 被外部改壞為無法解析後使用者開啟設定頁
- **THEN** config.yaml 簽浮出解析失敗說明，專案說明卡與產出規則卡的編輯鈕停用，不提供任何寫入途徑

---
### Requirement: 系統匣樣式偏好
於 macOS，設定頁 SHALL 提供「系統匣樣式」偏好（「原生選單」／「面板」二選），預設 SHALL 為原生選單；切換 SHALL 即時對系統匣生效（無需重啟）並持久化於 app 本機。app 本機偏好缺此值或值非法時 SHALL 視為原生選單（向後相容：舊安裝升級後行為不變、不浮出錯誤）。非 macOS 平台設定頁 SHALL NOT 顯示此偏好，系統匣固定為原生選單。此偏好 SHALL NOT 寫入 .speclink.yaml 或 openspec/config.yaml，兩檔內容 SHALL NOT 因切換而改動。

#### Scenario: 切換即時生效並持久化
- **WHEN** 使用者於 macOS 設定頁將系統匣樣式由「原生選單」切為「面板」
- **THEN** 未重啟 app 的情況下點擊系統匣圖示即改為彈出面板，重啟 app 後仍為面板樣式，且 .speclink.yaml 與 openspec/config.yaml 內容未被此操作改動

#### Scenario: 舊安裝缺此偏好視為原生選單
- **WHEN** app 於 app 本機偏好不含系統匣樣式值的狀態下啟動
- **THEN** 偏好讀取成功、無錯誤浮出，系統匣以原生選單樣式運作

#### Scenario: 非 macOS 平台不顯示此偏好
- **WHEN** 使用者於 Windows 或 Linux 開啟設定頁
- **THEN** 設定頁不出現「系統匣樣式」偏好，系統匣維持原生選單

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
# desktop-app Specification

## Purpose

TBD - created by archiving change 'desktop-shell-and-browser'. Update Purpose after archive.

## Requirements

### Requirement: 桌面 app 直嵌引擎並以本地檔案為真相
桌面 app SHALL 以 Tauri 殼直接內嵌 speclink-core（非 spawn CLI 子進程），於本地 openspec/ 專案根運作，且 SHALL NOT 改變 fs 模式下 markdown 檔案的真相地位——所有呈現資料由內嵌 core 讀取檔案取得，app 不將任何 change/spec 文件的真相移出檔案系統。

#### Scenario: 於 fs 專案根啟動並讀取本地文件
- **WHEN** 使用者於含 openspec/ 的專案根啟動桌面 app
- **THEN** app 經內嵌 core 讀取本地 markdown 顯示 change 與 spec，未 spawn speclink CLI 子進程，且未寫入或搬移任何文件真相

#### Scenario: 非專案目錄啟動顯示空狀態而非崩潰
- **WHEN** 使用者於不含 speclink 專案標記的目錄啟動桌面 app
- **THEN** app 顯示明確的「非 speclink 專案」空狀態，不崩潰、不產生錯誤彈窗堆疊


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容
桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致。

#### Scenario: 顯示 change 清單與狀態
- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容
- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 顯示其正典內容
- **WHEN** 使用者選定一個 spec
- **THEN** app 顯示該 spec 的正典 spec.md 內容


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 桌面 app 提供動詞操作面
桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

#### Scenario: 執行 validate 呈現結果
- **WHEN** 使用者對一個 change 觸發 validate
- **THEN** app 呈現與 speclink validate 一致的通過或失敗結果，失敗時顯示其錯誤訊息

#### Scenario: 執行 analyze 呈現發現項
- **WHEN** 使用者對一個 change 觸發 analyze
- **THEN** app 呈現 analyze 的發現項，其嚴重度與訊息對應 speclink analyze 的 --json 輸出

#### Scenario: archive 前置未滿足時失敗顯示
- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 歸檔清單經衍生快取加速且可重建
桌面 app SHALL 以本地 SQLite 索引快取歸檔（archived）change 的清單資料以加速呈現；此快取 SHALL 為衍生資料——可刪除後由檔案系統真相重建，且 SHALL 帶 schema 版本標記。active change 與 spec 的清單 SHALL NOT 依賴此快取，一律即時經 core 讀取檔案。快取與檔案真相不一致時，檔案真相 SHALL 為準。

#### Scenario: 歸檔清單由快取呈現
- **WHEN** app 呈現歸檔 change 清單
- **THEN** 清單資料自 SQLite 快取讀取，內容與檔案系統中的歸檔目錄一致

#### Scenario: 快取遺失時重建
- **WHEN** SQLite 快取檔不存在或版本不符
- **THEN** app 由歸檔目錄重建快取後呈現清單，不崩潰

#### Scenario: active 清單不經快取
- **WHEN** app 呈現 active change 清單
- **THEN** 清單即時經 core 讀取檔案取得，不讀取 SQLite 快取


<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 前端元件庫與資料源解耦
桌面 app 的呈現元件（change 看板、文件樹、文件檢視）SHALL 封裝為與資料源解耦的共用元件庫：元件經注入的 data adapter 取得資料，adapter 介面 SHALL 以領域語彙（列出 change、列出 spec、取得文件、執行動詞）定義，SHALL NOT 直接依賴 Tauri 專屬全域。桌面 app SHALL 提供以內嵌 core 為後端的 adapter 實作。

#### Scenario: 桌面注入 core adapter 渲染看板
- **WHEN** 桌面 app 以其 core-backed adapter 提供 change 列表
- **THEN** 共用看板元件據此渲染，元件本身未引用任何 Tauri 專屬全域

#### Scenario: adapter 介面以領域語彙定義
- **WHEN** 檢視 adapter 介面定義
- **THEN** 其方法以 change/spec/document/verb 領域語彙表述，使非 Tauri 後端（如後續 HTTP 後端）可提供另一實作而元件不變

<!-- @trace
source: desktop-shell-and-browser
updated: 2026-07-05
code:
  - CLAUDE.md
  - Cargo.lock
  - Cargo.toml
  - apps/desktop/core/Cargo.toml
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/index.html
  - apps/desktop/package.json
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/build.rs
  - apps/desktop/src-tauri/capabilities/default.json
  - apps/desktop/src-tauri/icons/icon.ico
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - apps/desktop/src/store.ts
  - apps/desktop/tsconfig.json
  - apps/desktop/vite.config.ts
  - apps/desktop/vitest.config.ts
  - package-lock.json
  - package.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/__tests__/ui.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeBoard.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DetailDrawer.tsx
  - packages/ui/src/components/DocumentTree.tsx
  - packages/ui/src/components/DocumentViewer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/alert-dialog.tsx
  - packages/ui/src/components/ui/badge.tsx
  - packages/ui/src/components/ui/button.tsx
  - packages/ui/src/components/ui/card.tsx
  - packages/ui/src/components/ui/input.tsx
  - packages/ui/src/components/ui/sheet.tsx
  - packages/ui/src/components/ui/tabs.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/lib/utils.ts
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
  - packages/ui/tsconfig.json
  - packages/ui/vitest.config.ts
-->

---
### Requirement: 外部變更即時反映

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情抽屜、討論抽屜與已封存頁 SHALL 在短時間內（秒級）自動更新呈現；已開啟檢視中「已載入的內容」——任務清單勾選狀態、proposal／design／specs 文件原文、meta 開工歸屬、討論記錄各分頁——SHALL 重載至磁碟現況，SHALL NOT 要求重啟、重開抽屜或任何 app 內操作。使用者互動進行中（任務勾選寫回、拖曳排序）時外部觸發的內容重載 SHALL 讓路，互動結束後 SHALL 補一次重載至磁碟現況——SHALL NOT 打斷或蓋掉進行中的操作；重載回應交錯時 SHALL 以最新一次為準，較舊回應 SHALL NOT 覆蓋較新內容。app 內操作（勾任務、拖曳、動詞）後 SHALL 重載受影響的已載入內容——含任務清單與 meta。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新；抽屜若開啟，標頭計數與任務清單中該項的核取方塊皆同步至磁碟狀態，全程無任何 app 內操作

#### Scenario: 外部蓋開工章後抽屜出現開工歸屬

- **WHEN** 某 change 的詳情抽屜開啟中，於外部終端執行 speclink in-progress add 該 change
- **THEN** 數秒內抽屜出現開工者與開工日，無需重開抽屜

#### Scenario: 外部推進討論後抽屜內容更新

- **WHEN** 某討論的抽屜開啟中，於外部終端執行 speclink discuss add-round 該討論
- **THEN** 數秒內抽屜的回合分頁出現新回合內容，標頭回合數與其一致

#### Scenario: 互動進行中外部重載讓路

- **WHEN** 使用者正在拖曳某 change 的任務（尚未放開），外部寫者同時修改該 change 的文件
- **THEN** 拖曳互動不被打斷、拖曳視覺不重置；放開完成後數秒內，抽屜內容重載至磁碟現況

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊


<!-- @trace
source: drawer-live-reload
updated: 2026-07-07
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/store.ts
  - packages/ui/src/__tests__/changeListItem.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/ChangeListItem.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: 已封存變更可展開檢視

已封存頁的每列 SHALL 顯示日期、名稱與任務數徽章，且 SHALL 可展開為唯讀檢視——至少含提案、設計、任務、規格分頁，內容來自封存目錄的實體文件。檢視 SHALL 為唯讀：SHALL NOT 提供任務勾選、動詞執行或任何寫入操作。所請求的文件不存在時對應分頁 SHALL 顯示空狀態而非錯誤。

#### Scenario: 展開封存列檢視文件

- **WHEN** 使用者於已封存頁點擊一個含完整 artifacts 的封存列
- **THEN** 列展開顯示提案／設計／任務／規格分頁，各分頁呈現封存目錄內對應文件的內容，任務分頁的核取方塊不可點擊

#### Scenario: 徽章顯示任務計數

- **WHEN** 已封存頁載入一個 tasks.md 為 48 項全勾的封存變更
- **THEN** 該列顯示 48/48 徽章；無 tasks.md 的封存變更不顯示徽章

#### Scenario: 缺件文件顯示空狀態

- **WHEN** 使用者展開一個無 design.md 的封存變更並切至設計分頁
- **THEN** 分頁顯示空狀態文字，無錯誤彈窗，其餘分頁照常可用

<!-- @trace
source: desktop-board-parity
updated: 2026-07-06
code:
  - .spectra.yaml
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/testfixture.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 看板欄位由生命週期標記驅動

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at 或任務完成數大於 0＝進行中；其餘＝提案中。剛完成 propose（有任務、全未勾、未標記開工）的 change SHALL 顯示於提案中欄。詳情抽屜 SHALL 於 meta 含 started_at 時顯示開工者與開工日（started_by、started_at）；meta 無 started_at 時 SHALL NOT 顯示開工資訊——即使該 change 因任務進度列於進行中欄（派生管顯示，歸屬缺席維持缺席）。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，抽屜標頭顯示開工者與開工日

#### Scenario: 無章而有任務進度列於進行中

- **WHEN** 某 change 的 meta 無 started_at，其 tasks.md 經任意途徑（如編輯器直接修改後 git pull 或本機儲存）成為 3/28 已勾，看板刷新
- **THEN** 該卡片顯示於「進行中」欄，詳情抽屜不顯示開工者與開工日

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 無 | 3/28 | 進行中（抽屜無開工資訊） |
| 有 | 0/28 | 進行中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |


<!-- @trace
source: task-done-implies-started
updated: 2026-07-07
code:
  - Cargo.lock
  - apps/desktop/core/src/manage.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: 任務清單拖放排序與自動重編號

詳情抽屜任務分頁的每個任務列 SHALL 於列首提供拖曳把手，使用者拖曳把手放開後 SHALL 一次到位完成排序並回寫 tasks.md——僅 checkbox 行被搬移，群組標題與其他行 SHALL NOT 被搬動。寫回時 SHALL 重寫受影響任務的編號前綴：文字以「數字.數字」＋空白開頭、且位於帶「數字.」前綴標題之群組下的任務行，其前綴 SHALL 重寫為「群組編號.組內序」；不符該樣式的任務行與所有非 checkbox 行 SHALL 逐字元保留。拖曳 SHALL 僅由把手觸發——點擊核取方塊或任務文字 SHALL NOT 啟動拖曳。群組標題 SHALL 參與拖曳中的讓位序列——被越過任務的讓位視覺 SHALL NOT 穿越其群組標題（群組歸屬在拖曳預覽中保持可讀）；群組標題為「組界槽」落點——把手放開於群組標題上時：來自標題上方的任務 SHALL 成為該群組的第一個任務；來自標題下方的任務 SHALL 移到標題之前、成為上一群組的末任務（雙向皆重編號；標題該側無任務可錨定時不提供落點）。逐格上下移動按鈕 SHALL NOT 再提供。封存唯讀檢視 SHALL NOT 渲染把手。

#### Scenario: 組內拖放後編號重寫

- **WHEN** 使用者把群組 1 的首任務拖到該群組末位放開
- **THEN** tasks.md 該群組的 checkbox 行依新順序排列，編號前綴依序重寫為 1.1、1.2、…（被拖任務取得末位編號），群組標題與其他群組逐字元不變

##### Example: 組內移動重編號

| 拖放前（群組 1） | 拖放後（把 1.1 甲拖到末位） |
| ---------------- | --------------------------- |
| - [ ] 1.1 甲 | - [x] 1.1 乙 |
| - [x] 1.2 乙 | - [ ] 1.2 丙 |
| - [ ] 1.3 丙 | - [ ] 1.3 甲 |

#### Scenario: 跨群組拖放取得新群組編號

- **WHEN** 使用者把群組 1 的末任務拖到群組 2 首任務之上放開（落點為該任務）
- **THEN** 該任務插於其後、前綴重寫為 2.2，群組 2 其餘任務依序後移（2.3…），群組 1 剩餘任務依序重排（1.1、1.2…）

#### Scenario: 拖放到群組標題成為組首

- **WHEN** 使用者把群組 1 的任務拖放到「## 2.」群組標題上
- **THEN** 該任務成為群組 2 的第一個任務（前綴重寫為 2.1），原組首任務後移為 2.2，群組 1 剩餘任務依序重排

##### Example: 標題落點成組首

| 拖放前 | 拖放後（把 1.2 乙拖到「## 2. 後段」標題上） |
| ------ | ------------------------------------------- |
| ## 1. 前段：1.1 甲、1.2 乙 | ## 1. 前段：1.1 甲 |
| ## 2. 後段：2.1 丙、2.2 丁 | ## 2. 後段：2.1 乙、2.2 丙、2.3 丁 |

#### Scenario: 組首任務拖回標題之前回到上一群組末位

- **WHEN** 使用者把群組 2 的第一個任務（原自群組 1 拖入）拖放到「## 2.」群組標題上
- **THEN** 該任務移到標題之前、成為群組 1 的末任務並重編號（如群組 1 原有 5 項則成為 1.6），群組 2 其餘任務依序前移補位

##### Example: 標題落點雙向

| 操作 | 結果 |
| ---- | ---- |
| 把 1.2 乙拖到「## 2. 後段」標題上（來自上方） | 乙成為 2.1 組首 |
| 再把 2.1 乙拖回「## 2. 後段」標題上（來自下方） | 乙回到群組 1 末位（1.2） |

#### Scenario: 拖曳讓位不穿越群組標題

- **WHEN** 使用者拖曳任務向下越過相鄰群組的首任務（拖曳進行中、尚未放開）
- **THEN** 該首任務的讓位視覺仍位於其群組標題之下方——SHALL NOT 出現在標題上方（上一群組的區域內）

#### Scenario: 無編號前綴的任務文字不被改寫

- **WHEN** 使用者拖動一個文字不以「數字.數字」開頭的任務到新位置
- **THEN** 該任務僅位置改變、文字逐字元保留，其他符合樣式的任務仍正常重編號

#### Scenario: 點擊核取方塊不誤觸拖曳

- **WHEN** 使用者在任務核取方塊上按下並於拖曳啟動閾值內放開
- **THEN** 勾選狀態切換且任務順序不變，無拖曳視覺出現

#### Scenario: 封存唯讀檢視無把手

- **WHEN** 使用者展開已封存變更的任務分頁
- **THEN** 任務列無拖曳把手、無法拖動，核取方塊維持不可點擊

<!-- @trace
source: desktop-task-drag-reorder
updated: 2026-07-06
code:
  - .spectra.yaml
  - AGENTS.md
  - CLAUDE.md
  - Cargo.lock
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/testfixture.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-fs/tests/store_fs.rs
  - crates/speclink-node/src/store_bridge.rs
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/stage.ts
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 監看根解析與專案探索一致

桌面 app 的檔案監看 SHALL 依附於經專案探索解析出的專案根：啟動時 SHALL 自工作目錄向上探索 speclink 專案（與查詢指令同一探索邏輯、同一結果），並監看該專案實際的 spec 目錄樹；SHALL NOT 以未經探索的啟動工作目錄拼接固定目錄名作為監看目標。以非專案根的工作目錄啟動（如自檔案總管雙擊執行檔）時，外部變更的自動刷新 SHALL 照常生效。向上探索不到任何 speclink 專案時 SHALL 維持既有降級行為：app 照常提供其餘功能、僅失去自動刷新、錯誤記錄於日誌。

#### Scenario: 自非專案根 cwd 啟動後外部開工即時反映

- **WHEN** 桌面 app 以專案內子目錄（如建置輸出目錄）為工作目錄啟動，隨後於外部終端對某 change 標記開工（其 metadata 文件寫入 started_at）
- **THEN** 數秒內看板該卡片自「提案中」欄移至「進行中」欄，全程無重啟或任何 app 內操作

#### Scenario: 監看目標尊重非預設 spec 目錄名

- **WHEN** 專案使用非預設的 spec 目錄名，桌面 app 於該專案內啟動，外部寫者修改該 spec 目錄下的文件
- **THEN** 看板與詳情於數秒內自動更新——監看目標為探索出的實際 spec 目錄，而非寫死的目錄名

#### Scenario: 探索不到專案時維持降級行為

- **WHEN** 桌面 app 於任何 speclink 專案之外的目錄啟動
- **THEN** app 照常啟動與運作，僅無自動刷新；監看建立失敗只記錄於日誌，畫面無錯誤彈窗

<!-- @trace
source: desktop-watcher-root-fix
updated: 2026-07-06
code:
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/watch.rs
-->

---
### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，依討論狀態兩級呈現：status 為 open 或 concluded 的討論 SHALL 為全尺寸卡（顯示 topic、輪數與狀態）——open 卡為唯讀，concluded 卡 SHALL 提供「轉為變更」與「封存」動詞；status 為 promoted 的討論 SHALL 收合於欄底「已轉出變更的討論」群組的細列——細列 SHALL 以討論 topic 為首行（slug SHALL NOT 出現於看板），其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段標示。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪）與一筆 status: concluded 的討論
- **THEN** 討論欄顯示兩張全卡：open 卡呈現 topic 與「3 輪」且無動詞按鈕，concluded 卡帶「轉為變更」與「封存」按鈕

#### Scenario: 已轉出討論收合為衍生樹細列

- **WHEN** 一筆 topic 為「桌面即時刷新與封存瀏覽」的討論，其 promoted_to 含兩個變更，其一在 active 清單（有任務未開工）、其一已在封存清單
- **THEN** 該討論不以全卡呈現，而在「已轉出變更的討論」群組顯示一列：首行為該 topic，其下兩列樹狀子項——前者帶 ├ 前綴標示提案中、後者帶 └ 前綴標示已封存

##### Example: chip 階段派生矩陣

| promoted_to 子變更的所在 | 階段標示 |
| ------------------------ | -------- |
| active 清單，無 started、0/24 | 提案中 |
| active 清單，有 started、13/24 | 進行中 |
| active 清單，24/24 | 已就緒 |
| 封存清單（dated name 尾碼命中） | 已封存 |
| 兩清單皆無 | 已刪除（討論維持已轉出） |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作


<!-- @trace
source: desktop-discussion-ui-polish
updated: 2026-07-06
code:
  - .spectra.yaml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/teststore.rs
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 討論抽屜檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論抽屜。抽屜標題下方 SHALL 呈現生命週期階梯「討論中 → 已結論 → 轉出變更」且現站可辨識。分頁 SHALL 依序為：結論、討論過程 N、背景、衍生變更——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，並於 concluded 與 promoted 狀態提供轉出動詞——尚未轉出時按鈕文字為「轉為變更」、已轉出過為「再轉出一個變更」。轉為變更 SHALL 經確認後建立新變更——其 meta 含 from_discussion、proposal 以討論結論預填——並使新卡現身提案中欄、討論的 promoted_to 累積該名稱；確認框說明 SHALL 以使用者語言描述後果（新增變更卡、提案以結論開頭、討論移入已轉出區），SHALL NOT 出現 from_discussion、kebab-case 等工程詞，名稱輸入說明 SHALL 為「英文小寫，字間用 -」。concluded 卡的封存動詞 SHALL 經確認後將討論移入封存。轉為變更失敗（同名變更已存在、討論已封存等）SHALL 顯示單行錯誤且看板不變。GUI SHALL NOT 提供 conclude、add-round、new、discard——討論的推進與結論撰寫屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的抽屜
- **THEN** 抽屜顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；階梯顯示「已結論」為現站

#### Scenario: GUI 轉為變更建立變更

- **WHEN** 使用者於已結論討論卡按「轉為變更」並確認
- **THEN** 新變更出現於提案中欄、其 .openspec.yaml 含 from_discussion、proposal.md 以結論預填；討論改於「已轉出變更的討論」群組以細列呈現且 promoted_to 含新變更名

#### Scenario: 再轉出一個變更（扇出第二刀）

- **WHEN** 使用者於已轉出討論的抽屜衍生變更分頁按「再轉出一個變更」、輸入新名稱並確認
- **THEN** 第二個變更建立並現身提案中欄，細列樹狀子項增加對應一列，promoted_to 累積兩個名稱

#### Scenario: 轉為變更失敗浮出錯誤

- **WHEN** 轉出的變更名與既有 active 變更同名
- **THEN** 前端顯示單行錯誤訊息，看板與討論記錄皆不變

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源變更清單，點擊同源項可開啟該變更的詳情


<!-- @trace
source: desktop-discussion-ui-polish
updated: 2026-07-06
code:
  - .spectra.yaml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/teststore.rs
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 已封存頁含討論節

已封存頁 SHALL 分「變更」與「討論」兩節：變更節維持既有展開列；討論節 SHALL 列出封存討論（日期＋topic）並可展開唯讀檢視記錄內容，SHALL NOT 提供任何寫入動詞。搜尋框 SHALL 同時過濾兩節。隨最後一個子變更歸檔而自動封存的討論、與經 GUI 或 CLI 手動封存的討論 SHALL 一致地出現於此節。展開檢視的區段標題 SHALL 使用「背景」「討論過程」「結論」。

#### Scenario: 封存討論唯讀展開

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論
- **THEN** 列展開顯示記錄內容（背景、討論過程、結論），無轉為變更、封存或任何寫入按鈕

#### Scenario: 自動封存的討論現身討論節

- **WHEN** 某已轉出討論的最後一個子變更被歸檔（觸發引擎的討論自動封存）且看板更新
- **THEN** 該討論自看板討論欄消失，已封存頁討論節出現該筆，搜尋其 topic 可命中


<!-- @trace
source: desktop-discussion-ui-polish
updated: 2026-07-06
code:
  - .spectra.yaml
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/teststore.rs
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: GUI 勾任務與 CLI 完成語意一致

桌面看板勾選任務為完成 SHALL 產生與 speclink task done 相同的檔案效果：tasks.md 該任務勾章、touched-files 記錄（有未被先前任務認領的 git dirty 檔時追加，無則不追加——與 CLI 同語意）、該 change 首次有任務完成且 meta 無 started_* 時蓋開工章（started_at 為當日、started_by 依 git 身分可得性、started_with 缺席）。對已完成任務的重複完成請求 SHALL 視為冪等成功，SHALL NOT 寫入任何檔案、SHALL NOT 對使用者報錯。取消勾選與拖曳排序 SHALL 僅寫 tasks.md，SHALL NOT 寫入 meta 或 touched 記錄。

#### Scenario: 勾選首任務蓋章並移欄

- **WHEN** 使用者於看板勾選某 meta 無 started_* 的 change 的第一項任務
- **THEN** tasks.md 該任務成 [x]，.openspec.yaml 新增 started_at（git 身分可得時含 started_by），看板刷新後卡片移入「進行中」欄且抽屜顯示開工列

#### Scenario: 取消勾選不動 meta 與 touched

- **WHEN** 使用者取消勾選一項已完成任務
- **THEN** 僅 tasks.md 該行標記變為 [ ]；.openspec.yaml 與 touched 記錄逐字元不變

#### Scenario: 拖曳排序不觸發完成語意

- **WHEN** 使用者拖曳任務改變順序（含跨群組重編號）
- **THEN** 僅 tasks.md 變動；.openspec.yaml 與 touched 記錄逐字元不變

<!-- @trace
source: task-done-implies-started
updated: 2026-07-07
code:
  - Cargo.lock
  - apps/desktop/core/src/manage.rs
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/tests/no_direct_fs.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/stage.test.ts
  - packages/ui/src/stage.ts
-->

---
### Requirement: 看板搜尋過濾卡片

看板 SHALL 於欄位上方提供搜尋輸入。輸入含非空白內容時，看板 SHALL 僅顯示比對命中的卡片——變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；比對 SHALL 去除頭尾空白、不分大小寫、以子字串命中（與已封存頁的搜尋規則一致）。各欄欄頭計數 SHALL 等於該欄過濾後的卡片數。清空輸入（或輸入僅含空白）SHALL 還原全量呈現。搜尋字串 SHALL NOT 持久化，且 SHALL 與已封存頁的搜尋字串各自獨立。

#### Scenario: 輸入關鍵字即時過濾各欄卡片

- **WHEN** 使用者於看板搜尋輸入鍵入非空白字串
- **THEN** 各欄僅顯示名稱或摘要（變更卡）、主題或 slug（討論卡）以不分大小寫子字串命中的卡片，且各欄欄頭計數等於該欄過濾後卡片數

##### Example: 比對規則

- **GIVEN** 提案中欄有變更卡 desktop-acp-agent（摘要含「桌面版」）與 web-role-views（摘要含「情境 1」）；討論欄有卡片主題「GUI 勾任務自動蓋開工章」

| 輸入 | 提案中欄顯示（計數） | 討論欄顯示（計數） | Notes |
| ---- | -------------------- | ------------------ | ----- |
| desktop | desktop-acp-agent（1） | 無（0） | 名稱子字串命中 |
| 桌面 | desktop-acp-agent（1） | 無（0） | 摘要命中 |
| &nbsp;GUI&nbsp; | 無（0） | GUI 勾任務自動蓋開工章（1） | 去頭尾空白、不分大小寫 |
| （清空） | 兩張全顯（2） | 全顯（1） | 還原全量 |

#### Scenario: 無命中時顯示空欄與零計數

- **WHEN** 使用者輸入無任何卡片命中的字串
- **THEN** 各欄顯示為空、欄頭計數為 0，欄位結構維持呈現且不顯示錯誤

#### Scenario: 過濾狀態下卡片互動不受影響

- **WHEN** 過濾狀態下使用者點擊卡片、或拖曳已就緒的變更卡至封存落點
- **THEN** 詳情抽屜正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空

<!-- @trace
source: desktop-board-search
updated: 2026-07-07
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/store.ts
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/search.ts
-->

---
### Requirement: 側欄導覽結構

桌面 app 側欄 SHALL 呈現四個導覽項：變更、規格、已封存 SHALL 由上而下依序位於側欄頂部；設定 SHALL 固定於側欄底部，與頂部群組之間為彈性空間。側欄 SHALL NOT 含備忘項（其 i18n 鍵自兩語系字典移除，zh-TW 與 en 鍵集合維持相等）。已封存導覽項 SHALL 帶封存變更數量徽章，數字 SHALL 隨封存清單變動即時更新；其無障礙標籤 SHALL 為「已封存」。導覽項點擊 SHALL 為單純切頁——切至對應頁並以視覺高亮標示現行項，SHALL NOT 具再次點擊返回看板的 toggle 行為；設定項沉底 SHALL NOT 改變其高亮與切頁語意。頂欄 SHALL NOT 含已封存入口。

#### Scenario: 側欄順序與設定沉底

- **WHEN** 開啟任一專案進入桌面 app
- **THEN** 側欄頂部由上而下依序為變更、規格、已封存，設定獨立位於側欄底部（兩者之間為空白彈性區），不存在備忘項，頂欄不存在已封存鈕

#### Scenario: 已封存徽章隨封存動作更新

- **WHEN** 看板上一個已就緒的變更被封存
- **THEN** 已封存導覽項的徽章數量在數秒內增加一、無需重啟或手動重新整理

#### Scenario: 已封存導覽項為切頁而非 toggle

- **WHEN** 使用者位於已封存頁並再次點擊已封存導覽項
- **THEN** 畫面停留在已封存頁；隨後點擊變更導覽項才返回看板，且變更項高亮、已封存項恢復未選取樣式

---
### Requirement: 視窗預設尺寸與置中

桌面 app 視窗 SHALL 以 1440×900 邏輯尺寸啟動，並 SHALL 於主螢幕置中。此為靜態預設值：app SHALL NOT 記憶上次視窗大小與位置，每次啟動皆以相同預設呈現。

#### Scenario: 啟動視窗尺寸與置中

- **WHEN** 使用者啟動桌面 app
- **THEN** 視窗邏輯尺寸為 1440×900，且視窗於主螢幕水平與垂直置中

##### Example: 1920×1080 主螢幕下的位置

- **GIVEN** 主螢幕解析度 1920×1080、縮放 100%
- **WHEN** 啟動桌面 app
- **THEN** 視窗尺寸 1440×900，左右邊距各約 240、上下邊距各約 90（邏輯像素，實際垂直置中依作業系統工作列語意由視窗系統決定）

---
### Requirement: 介面文字以打包的 Noto Sans TC 呈現

桌面 app SHALL 將 Noto Sans TC 字體隨應用程式打包，並以其為介面與內容文字的第一優先字體；未安裝該字體的機器與離線環境 SHALL 呈現相同字體，SHALL NOT 依賴網路下載字體資產。等寬文字（inline code 與程式碼區塊）SHALL 維持等寬字體，不受此變更影響。

#### Scenario: 未安裝字體的機器呈現打包字體

- **WHEN** 在未安裝 Noto Sans TC 的作業系統上啟動桌面 app 並開啟任一文件檢視
- **THEN** 介面與 markdown 內容文字以打包的 Noto Sans TC 呈現，無對外字體網路請求

#### Scenario: 等寬文字不受字體變更影響

- **WHEN** 檢視含 inline code 或程式碼區塊的文件
- **THEN** 該段文字以等寬字體呈現，與周圍的 Noto Sans TC 內文明顯可辨

<!-- @trace
source: desktop-reading-experience
updated: 2026-07-08
code:
  - apps/desktop/package.json
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: markdown 內容保留文件結構呈現

桌面 app 渲染 markdown 內容（變更抽屜的提案／設計／規格分頁、討論抽屜各分頁、已封存檢視）SHALL 保留來源的文件結構：無序清單 SHALL 顯示列表符號、有序清單 SHALL 顯示編號、段落之間 SHALL 有可辨識的垂直間距、來源中的單一換行 SHALL 呈現為換行。內容基準字級 SHALL 為 16px，任務分頁的任務文字 SHALL 同為 16px。排版與內容色彩 SHALL 於淺色與深色主題（跟隨系統偏好）一致生效。

#### Scenario: 清單顯示列表符號與編號

- **WHEN** 檢視含無序清單與有序清單的提案文件
- **THEN** 無序清單項目前顯示列表符號、有序清單項目前顯示編號，清單相對內文有縮排

#### Scenario: 單一換行呈現為換行

- **WHEN** 檢視討論記錄的討論過程分頁，其中一輪的 Focus 與 Position 行在來源中各佔一行、以單一換行分隔
- **THEN** 兩行在渲染結果中分行呈現，不塌成同一段連續文字

#### Scenario: 內容基準字級為 16px

- **WHEN** 檢視變更抽屜的提案分頁與任務分頁
- **THEN** markdown 內文與任務清單文字的計算字級皆為 16px

#### Scenario: 深色主題下排版一致

- **WHEN** 系統偏好為深色時檢視同一份文件
- **THEN** 列表符號、段落間距、字級與淺色主題一致，內容色彩取自深色 token 且可讀

<!-- @trace
source: desktop-reading-experience
updated: 2026-07-08
code:
  - apps/desktop/package.json
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/TaskList.tsx
-->

---
### Requirement: raw HTML 不以原文呈現

桌面 app 渲染 markdown 內容時，來源中的 raw HTML（含 HTML 註解）SHALL NOT 以原始文字出現在渲染結果；渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 討論記錄的 scaffold 註解不顯示

- **WHEN** 開啟討論抽屜的討論過程分頁，其來源在 Rounds 區段含 CLI scaffold 產生的 HTML 註解行
- **THEN** 渲染結果不含該註解的任何文字，openspec/ 下的討論記錄檔案內容位元不變

#### Scenario: 程式碼區塊內的 HTML 原文照常顯示

- **WHEN** 檢視在 code fence 內含 HTML 標籤範例的文件
- **THEN** code fence 內容以原文完整顯示（過濾僅及於 code fence 之外的 raw HTML）

<!-- @trace
source: desktop-reading-experience
updated: 2026-07-08
code:
  - apps/desktop/package.json
  - apps/desktop/src/index.css
  - apps/desktop/src/main.tsx
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/TaskList.tsx
-->
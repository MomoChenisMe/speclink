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

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情抽屜與已封存頁 SHALL 在短時間內（秒級）自動更新呈現，SHALL NOT 要求重啟或任何 app 內操作。app 內操作（勾任務、動詞）後的即時反映 SHALL 維持既有行為。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新，抽屜若開啟亦同步，全程無任何 app 內操作

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊

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

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at＝進行中；其餘＝提案中。剛完成 propose（有任務、未標記開工）的 change SHALL 顯示於提案中欄。詳情抽屜 SHALL 於 change 已開工時顯示開工者與開工日（started_by、started_at），未開工時不顯示該資訊。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，抽屜標頭顯示開工者與開工日

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |

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
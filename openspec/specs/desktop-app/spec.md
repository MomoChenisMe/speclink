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

桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。spec 的選定 SHALL 開啟唯讀規格抽屜——呈現正典 spec.md 全文與溯源資訊，寬度與全螢幕切換與變更詳情抽屜一致，開啟期間內容隨外部檔案變更反映；規格清單 SHALL NOT 提供行內展開。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致；自檔案系統衍生的呈現層輔助欄位（如 spec 的最後修改時間、需求數、Purpose 摘要、溯源變更數）不屬此對齊範圍，SHALL NOT 出現在 CLI 輸出對照要求中。

#### Scenario: 顯示 change 清單與狀態

- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容

- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 以抽屜顯示其正典內容

- **WHEN** 使用者於規格頁點擊一張規格卡
- **THEN** 開啟唯讀規格抽屜顯示該 spec 的正典 spec.md 全文與溯源資訊，清單卡片無行內展開，抽屜寬度與變更詳情抽屜一致


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
### Requirement: 桌面 app 提供動詞操作面

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

詳情抽屜的動作列 SHALL 以單一「分析」按鈕同時觸發 validate 與 analyze，SHALL NOT 提供獨立的「驗證」按鈕；結果呈現於該 change 詳情抽屜內的分析面板，而非僅視窗頂列狀態列。分析面板 SHALL 依序呈現：

- 結構驗證列：validate 通過時呈單列通過標示（成功語意配色）；失敗時呈錯誤數並逐條列出錯誤訊息（與 speclink validate 的輸出一致）。
- 維度摘要卡：Coverage、Consistency、Ambiguity、Gaps 四維度各一張摘要卡，維度名以繁體中文呈現（覆蓋度、一致性、模糊度、缺漏），卡上顯示該維度發現數——零發現呈「無問題」（成功語意配色）、非零呈「N 個問題」（警示語意配色）。
- 發現卡：逐條發現各一張卡，呈現嚴重度徽章、來源檔（location）、摘要（summary）與建議行（recommendation），對應 speclink analyze 的 --json 輸出欄位。

分析面板 SHALL 可關閉：再次點按「分析」按鈕或面板的關閉鈕皆 SHALL 收合面板；收合後再點按「分析」SHALL 重新執行並展開。面板狀態 SHALL NOT 跨 change 沿用（切換 change 即清空）。視窗頂列狀態列 SHALL 保留供看板全域操作（刪除、封存、拖排失敗）之結果訊息。

#### Scenario: 分析一鍵呈現驗證列與四維度發現卡

- **WHEN** 使用者於某 change 的詳情抽屜點按「分析」
- **THEN** 抽屜內展開分析面板：頂部結構驗證列（通過或錯誤數）、四張繁體中文維度摘要卡（各帶發現數）、逐條發現卡（嚴重度徽章、來源檔、摘要、建議行），內容對應 speclink validate 與 speclink analyze 的 --json 輸出

##### Example: 維度摘要卡的呈現

- **GIVEN** analyze 回傳 Coverage 0、Consistency 0、Ambiguity 18、Gaps 0 個發現

| 維度卡 | 顯示 | 配色語意 |
| ------ | ---- | -------- |
| 覆蓋度 | 無問題 | 成功 |
| 一致性 | 無問題 | 成功 |
| 模糊度 | 18 個問題 | 警示 |
| 缺漏 | 無問題 | 成功 |

#### Scenario: 分析面板可收合

- **WHEN** 分析面板開啟後，使用者再次點按「分析」或點按面板關閉鈕
- **THEN** 面板收合；再次點按「分析」重新執行 validate 與 analyze 並展開面板

#### Scenario: 結構驗證失敗於分析面板呈現錯誤

- **WHEN** 使用者對結構驗證失敗的 change 點按「分析」
- **THEN** 結構驗證列呈現錯誤數並逐條列出 speclink validate 回報的錯誤訊息；維度摘要卡與發現卡照常呈現

#### Scenario: archive 前置未滿足時失敗顯示

- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔

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

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題（等寬字型）、topic 為卡身描述；複製 slug 鈕 SHALL 行內尾隨於標題最後一個字元後（版面規則見「看板卡片統一解剖學」）；建立者（createdBy，缺席時省略）SHALL 以頭像圓點呈現且 hover 顯示全名；輪數與建立時間 SHALL 並排於卡底 meta 列，狀態 chip 保留；open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）。

當存在至少一筆 promoted 討論時，欄底 SHALL 呈現「已轉出 N」收合列（N 為 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單，再點按即收合；展開狀態 SHALL NOT 跨啟動持久化。無任何 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 openspec/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示 active（open 與 concluded）數。當討論欄無任何 active 討論但存在 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by 與 created）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡，各以 slug 為標題（等寬字型）、topic 為描述，複製 slug 鈕行內尾隨於標題末字元後，建立者呈頭像圓點且 hover 顯示全名、卡面無全名直出文字，卡底 meta 列並排「3 輪」與建立時間；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，欄底無「已轉出」收合列

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡或已轉出細列的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋，且不開啟討論抽屜

#### Scenario: 欄底收合列就地展開與收合

- **WHEN** 討論欄存在一筆 active 討論與一筆 promoted 討論
- **THEN** 上區顯示 active 全卡、欄底呈現「已轉出 1」收合列（預設收合）；點按收合列就地展開 promoted 細列（active 全卡維持可見），再點按即收合；欄計數徽章全程顯示 1（active 數）

#### Scenario: 無已轉出討論時收合列缺席

- **WHEN** 討論欄無任何 promoted 討論
- **THEN** 欄底不顯示「已轉出」收合列

#### Scenario: 僅有已轉出討論時討論中區不顯空狀態

- **WHEN** 討論欄無任何 active（open 或 concluded）討論、但存在至少一筆 promoted 討論
- **THEN** 討論中區不顯示「尚無討論」文案，欄底收合列傳達存在已轉出討論

#### Scenario: 已轉出細列的 slug 首行與子變更樹配色

- **WHEN** 展開欄底收合列，一筆 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
- **THEN** 該細列首行為 slug（等寬字型）帶複製鈕、次行為 topic，其下兩列樹狀子項——前者帶 ├ 前綴與「提案中」chip（呈提案中欄的 teal 濃度）、後者帶 └ 前綴與「已封存」chip（中性色）

##### Example: chip 階段派生與配色矩陣

| promoted_to 子變更的所在 | 階段標示 | chip 配色 |
| ------------------------ | -------- | --------- |
| active 清單，無 started、0/24 | 提案中 | 提案中欄的 teal 濃度 |
| active 清單，有 started、13/24 | 進行中 | 進行中欄的 teal 濃度 |
| active 清單，24/24 | 已就緒 | 已就緒欄的 teal |
| 封存清單（dated name 尾碼命中） | 已封存 | 中性色 |
| 兩清單皆無 | 已刪除（討論維持已轉出） | destructive 加刪除線 |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作


<!-- @trace
source: board-card-anatomy
updated: 2026-07-11
code:
  - apps/desktop/core/src/query.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
-->

---
### Requirement: 討論抽屜檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論抽屜。抽屜標題 SHALL 以討論 slug 為題（等寬字型）且帶複製 slug 鈕，topic SHALL 為標題下方副標（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外）。標題區下方 SHALL 呈現生命週期階梯「討論中 → 已結論 → 轉出變更」且現站可辨識。分頁 SHALL 依序為：結論、討論過程 N、背景、衍生變更——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。concluded 卡的封存動詞 SHALL 經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 抽屜標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的抽屜並點按標題旁的複製鈕
- **THEN** 抽屜標題呈現該討論的 slug（等寬字型）、其下副標呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的抽屜
- **THEN** 抽屜顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；階梯顯示「已結論」為現站

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的抽屜衍生變更分頁
- **THEN** 分頁列出各子變更現況與跳轉按鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論抽屜或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源變更清單，點擊同源項可開啟該變更的詳情

---
### Requirement: 已封存頁含討論節

已封存頁 SHALL 以頁內子頁籤呈現「變更」與「討論」兩節，預設顯示「變更」子頁籤；子頁籤標籤 SHALL 各帶該節目前（搜尋過濾後）筆數徽章。兩節皆為卡片清單，點擊卡片開啟唯讀抽屜（檢視行為見「已封存項目以抽屜檢視」）；討論節 SHALL 列出封存討論（日期＋topic），SHALL NOT 提供任何寫入動詞。搜尋框 SHALL 位於子頁籤之上並同時過濾兩節——身處任一子頁籤時，另一子頁籤的徽章 SHALL 反映其命中數。兩子頁籤的換頁頁碼 SHALL 互相獨立。討論清單資料缺席（向後相容路徑）時子頁籤列 SHALL 缺席、僅顯示變更清單。隨最後一個子變更歸檔而自動封存的討論、與經 GUI 或 CLI 手動封存的討論 SHALL 一致地出現於討論節。抽屜檢視的區段標題 SHALL 使用「背景」「討論過程」「結論」。

#### Scenario: 切換子頁籤直達討論清單

- **WHEN** 已封存頁含 40 筆封存變更與 26 筆封存討論，使用者點「討論」子頁籤
- **THEN** 直接顯示封存討論清單第 1 頁（建立日期最新在前），無需捲過任何變更卡

#### Scenario: 搜尋時兩子頁籤徽章顯示各自命中數

- **WHEN** 使用者於「變更」子頁籤輸入僅命中 3 筆封存討論、0 筆封存變更的搜尋字串
- **THEN** 「變更」徽章顯示 0 且清單顯示無結果空狀態；「討論」徽章顯示 3，切換後即見該 3 筆

#### Scenario: 自動封存的討論現身討論節

- **WHEN** 某已轉出討論的最後一個子變更被歸檔（觸發引擎的討論自動封存）且看板更新
- **THEN** 該討論自看板討論欄消失，已封存頁「討論」子頁籤出現該筆，搜尋其 topic 可命中


<!-- @trace
source: specs-archive-pagination
updated: 2026-07-12
code:
  - packages/ui/src/__tests__/archivedDrawer.test.tsx
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/listPager.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/specDrawer.test.tsx
  - packages/ui/src/__tests__/specList.test.tsx
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/ListPager.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SpecDrawer.tsx
  - packages/ui/src/components/SpecList.tsx
  - packages/ui/src/i18n.tsx
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

看板 SHALL 於欄位上方提供單列搜尋工具列：搜尋輸入（帶搜尋圖示）SHALL 填滿工具列剩餘寬度，其右側的篩選開關鈕與輸入框同列同高，SHALL NOT 佔用第二列版面。輸入含非空白內容時 SHALL 呈現清除鈕（點按清空字串且輸入保持聚焦）與即時命中數（過濾後各欄卡片總數）；快捷鍵（macOS 為 Cmd+F、其他平台為 Ctrl+F）SHALL 聚焦搜尋輸入。

輸入含非空白內容時，看板 SHALL 僅顯示比對命中的卡片。比對 SHALL 含三層，任一層命中即顯示：

- 欄位子字串：變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；去除頭尾空白、不分大小寫、以子字串命中（與已封存頁的搜尋規則一致）。
- 名稱層模糊比對：變更卡名稱與討論卡 slug 另以 subsequence 比對（查詢字元依序出現於目標即命中）；摘要與主題 SHALL NOT 套用模糊比對。
- 全文比對：非空查詢 SHALL 經桌面 core 的 workspace 全文查詢，以不分大小寫子字串比對 active 變更的 artifacts（提案、設計、任務、delta 規格）與 active 討論記錄全文；查詢以去抖觸發、回應以 latest-wins 收斂；查詢失敗 SHALL 靜默退回欄位比對，SHALL NOT 阻斷輸入或顯示錯誤。

欄位子字串命中 SHALL 於卡片上以高亮標示命中原文；全文命中的卡片 SHALL 於卡身呈現 snippet 行——命中的 artifact 名與命中前後文裁切、命中原文高亮（每卡取首個命中）；僅模糊命中而無連續子字串時 SHALL 顯示卡片但不高亮。

篩選 SHALL 收於篩選開關鈕之後（預設不佔版面）：點按開關 SHALL 於其下方彈出篩選面板，面板內呈三個篩選維度選單——建立者（active 變更與討論的 createdBy 去重清單）、建立時間（近 7 天、近 30 天、更早三擇一；變更卡與討論卡皆以 created 日期比對）、來源討論（promoted_to 非空的討論清單；選定後顯示該討論卡自身與來源討論含該 slug 的變更卡）。再點按開關、點擊面板外或按 Esc SHALL 關閉面板；關閉面板 SHALL NOT 清除已啟用的篩選（過濾持續生效）。各維度 SHALL 可於面板內單獨清除；存在啟用中篩選時面板 SHALL 提供全部清除、且開關鈕 SHALL 呈現啟用計數。多個維度與搜尋字串 SHALL 以 AND 交集過濾。

各欄欄頭計數 SHALL 等於該欄過濾後的卡片數。清空輸入（或輸入僅含空白）且無啟用篩選時 SHALL 還原全量呈現。搜尋字串與篩選狀態 SHALL NOT 持久化，且 SHALL 與已封存頁的搜尋字串各自獨立。

#### Scenario: 輸入關鍵字即時過濾各欄卡片

- **WHEN** 使用者於看板搜尋輸入鍵入非空白字串
- **THEN** 各欄僅顯示任一比對層命中的卡片，命中的欄位字段以高亮標示，且各欄欄頭計數等於該欄過濾後卡片數

##### Example: 比對規則

- **GIVEN** 提案中欄有變更卡 desktop-acp-agent（摘要含「桌面版」）與 web-role-views（摘要含「情境 1」）；討論欄有卡片主題「GUI 勾任務自動蓋開工章」

| 輸入 | 提案中欄顯示（計數） | 討論欄顯示（計數） | Notes |
| ---- | -------------------- | ------------------ | ----- |
| desktop | desktop-acp-agent（1） | 無（0） | 名稱子字串命中並高亮 |
| 桌面 | desktop-acp-agent（1） | 無（0） | 摘要命中並高亮 |
| &nbsp;GUI&nbsp; | 無（0） | GUI 勾任務自動蓋開工章（1） | 去頭尾空白、不分大小寫 |
| dta | desktop-acp-agent（1） | 無（0） | 名稱 subsequence 模糊命中、不高亮 |
| （清空） | 兩張全顯（2） | 全顯（1） | 還原全量 |

#### Scenario: 全文命中呈現 snippet

- **WHEN** 使用者輸入僅出現於某變更 design.md 內文（不出現於名稱與摘要）的字串
- **THEN** 該變更卡顯示且卡身呈現 snippet 行（命中 artifact 名＋命中前後文、命中原文高亮）；欄頭計數含該卡

#### Scenario: 全文查詢失敗靜默退回欄位比對

- **WHEN** workspace 全文查詢因 IPC 錯誤失敗
- **THEN** 看板維持欄位比對的過濾結果，不顯示錯誤訊息、搜尋輸入不受阻斷

#### Scenario: 篩選收於開關鈕的彈出面板

- **WHEN** 使用者點按篩選開關鈕，於面板啟用建立時間「近 7 天」後再點按開關關閉面板
- **THEN** 開啟前不呈現任何篩選控制；面板開啟時三個維度選單可見；關閉後面板消失、過濾持續生效且開關鈕呈現啟用計數 1

#### Scenario: 面板內全部清除

- **WHEN** 使用者於篩選面板啟用兩個維度後點按全部清除
- **THEN** 所有維度回到未啟用、看板還原（僅剩搜尋字串過濾）、開關鈕的啟用計數消失

#### Scenario: 篩選 chip 與搜尋字串交集過濾

- **WHEN** 使用者啟用建立時間「近 7 天」chip 且輸入非空白字串
- **THEN** 僅顯示 created 日期在近 7 天內且命中該字串的卡片；單獨清除該 chip 後回到僅以字串過濾

#### Scenario: 來源討論篩選

- **WHEN** 使用者於來源討論 chip 選定某 promoted 討論
- **THEN** 看板僅顯示該討論卡自身與來源討論含該 slug 的變更卡

#### Scenario: 快捷鍵聚焦與清除鈕

- **WHEN** 使用者按下 Cmd+F（macOS）或 Ctrl+F（其他平台）
- **THEN** 搜尋輸入取得聚焦；輸入非空時搜尋列呈現清除鈕與命中數，點清除鈕後字串清空、全量還原且輸入保持聚焦

#### Scenario: 無命中時顯示空欄與零計數

- **WHEN** 使用者輸入無任何卡片命中的字串
- **THEN** 各欄顯示為空、欄頭計數為 0，欄位結構維持呈現且不顯示錯誤

#### Scenario: 過濾狀態下卡片互動不受影響

- **WHEN** 過濾狀態下使用者點擊卡片、或拖曳已就緒的變更卡至封存落點
- **THEN** 詳情抽屜正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串與篩選不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串並啟用任一篩選 chip 後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空、無啟用中的篩選 chip

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

---
### Requirement: 任務分頁提供批次操作工具列

變更抽屜的任務分頁 SHALL 於任務清單頂部提供工具列：「全部已完成」SHALL 將該變更全部任務標為完成並以單次檔案寫入回寫 tasks.md；「重置任務」SHALL 將全部任務取消勾選並以單次檔案寫入回寫；「下一個未完成」SHALL 使第一個未完成任務捲入可視範圍並短暫高亮，n 快捷鍵 SHALL 等效。批次完成的開工章語意 SHALL 與逐一勾選一致——未開工變更首次完成任務時蓋開工章；重置 SHALL NOT 蓋開工章、SHALL NOT 記錄 touched。目標狀態已達成時重跑 SHALL 為冪等成功且不改檔。全部任務已完成時「全部已完成」與「下一個未完成」SHALL 呈現不可用；唯讀封存檢視 SHALL NOT 顯示工具列。

#### Scenario: 全部已完成單次寫回

- **WHEN** 對含未完成任務的變更按「全部已完成」
- **THEN** tasks.md 全部任務標為完成且僅發生單次檔案寫入，抽屜進度顯示 100%

##### Example: 批次完成的進度變化

| 操作前 | 操作 | 操作後 |
| ------ | ---- | ------ |
| 3/10 任務完成 | 全部已完成 | 10/10、進度 100% |
| 10/10 任務完成 | 全部已完成 | 不改檔（冪等）、按鈕不可用 |

#### Scenario: 重置任務不蓋開工章

- **WHEN** 對已有完成任務的變更按「重置任務」
- **THEN** tasks.md 全部任務取消勾選且僅發生單次檔案寫入，變更 meta 未新增開工章與 touched 記錄

#### Scenario: 批次完成沿用開工章語意

- **WHEN** 對尚未開工的變更按「全部已完成」
- **THEN** 變更 meta 蓋上開工章（與逐一勾選首次完成任務的行為一致）

#### Scenario: 下一個未完成定位

- **WHEN** 任務清單有多個任務且第一個未完成任務在可視範圍外，按「下一個未完成」或按 n 鍵
- **THEN** 該任務捲入可視範圍並短暫高亮

#### Scenario: 唯讀封存檢視不顯示工具列

- **WHEN** 於已封存頁展開封存變更的任務檢視
- **THEN** 不顯示批次操作工具列，checkbox 維持唯讀

<!-- @trace
source: desktop-task-interactions
updated: 2026-07-08
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - docs/assets/speclink-logo-concept.png
  - docs/assets/speclink-logo-horizontal.png
  - docs/assets/speclink-logo-lockup.png
  - docs/assets/speclink-logo-mark-redesign.png
  - docs/assets/speclink-logo-mark.png
  - docs/assets/speclink-logo-selected-lockup.png
  - docs/assets/speclink-logo-selected-mark.png
  - docs/assets/speclink-logo-system-sheet.png
  - docs/assets/speclink-logo-vertical.png
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 勾選任務即時回饋

任務分頁勾選或取消勾選任一任務時，畫面 SHALL 立即反映新勾選狀態（不等待寫回與重載完成），清單其餘任務 SHALL 保持可互動；寫回失敗時 SHALL 回滾至磁碟現況並顯示單行錯誤。寫回成功後的整批重載 SHALL 沿用既有 refresh 世代機制與互動讓路行為。

#### Scenario: 勾選立即反映

- **WHEN** 勾選一個未完成任務且寫回尚未完成
- **THEN** 該 checkbox 立即呈現勾選態與完成刪除線，無可感知等待

#### Scenario: 連續勾選不互鎖

- **WHEN** 快速連續勾選多個任務
- **THEN** 每次勾選皆立即反映，清單全程可互動，最終 tasks.md 與畫面一致

#### Scenario: 寫回失敗回滾

- **WHEN** 勾選任務後寫回失敗（如檔案不可寫）
- **THEN** 該 checkbox 回復原狀態並顯示單行錯誤訊息，tasks.md 內容不變

<!-- @trace
source: desktop-task-interactions
updated: 2026-07-08
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - docs/assets/speclink-logo-concept.png
  - docs/assets/speclink-logo-horizontal.png
  - docs/assets/speclink-logo-lockup.png
  - docs/assets/speclink-logo-mark-redesign.png
  - docs/assets/speclink-logo-mark.png
  - docs/assets/speclink-logo-selected-lockup.png
  - docs/assets/speclink-logo-selected-mark.png
  - docs/assets/speclink-logo-system-sheet.png
  - docs/assets/speclink-logo-vertical.png
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/tasks.ts
-->

---
### Requirement: 表單控制項與按鈕以主題化元件呈現

桌面 app 的互動控制項 SHALL 以元件庫的主題化原語呈現：勾選控制項（任務分頁勾選框、初始化對話框的工具多選）SHALL 呈現主題化外觀——未勾選為主題邊框空框、勾選為主色底加勾選圖示，SHALL NOT 呈現作業系統原生控制項外觀；多行文字輸入（設定頁）SHALL 取用設計 token 的邊框、底色與 focus 樣式；動作按鈕 SHALL 統一取用元件庫按鈕變體，鍵盤聚焦時 SHALL 顯示一致的 focus 可視環、disabled 態呈現一致。上述控制項於淺色與深色主題 SHALL 一致取自設計 token。替換 SHALL NOT 改變控制項的行為與無障礙語意：勾選框 SHALL 保留 checkbox 角色、既有標籤與空白鍵切換；按鈕 SHALL 保留既有無障礙名稱、回呼與 disabled 條件；唯讀封存檢視的勾選框 SHALL 維持不可互動。

#### Scenario: 任務勾選框主題化外觀

- **WHEN** 檢視含已完成與未完成任務的任務分頁
- **THEN** 未完成任務的勾選框為主題邊框空框、已完成為主色底加勾選圖示，非作業系統原生繪製，深淺主題下外觀一致

#### Scenario: 勾選框無障礙語意保留

- **WHEN** 以鍵盤或輔助技術操作任務分頁的勾選框
- **THEN** 每個勾選框曝露 checkbox 角色與「任務 N」標籤，空白鍵可切換勾選且觸發與滑鼠點擊相同的寫回

#### Scenario: 初始化對話框工具多選主題化

- **WHEN** 開啟初始化確認對話框
- **THEN** claude 與 codex 選項以主題化勾選框呈現且可獨立勾選，預設勾選狀態與替換前相同

#### Scenario: 設定頁多行輸入主題化

- **WHEN** 於深色主題開啟設定頁的專案說明或產出規則編輯區
- **THEN** 多行輸入的邊框、底色與 focus 樣式取自設計 token，輸入與儲存行為與替換前相同

#### Scenario: 按鈕 focus 可視環一致

- **WHEN** 以 Tab 鍵依序聚焦任務工具列、詳情抽屜動作列與側欄導覽的按鈕
- **THEN** 每個按鈕顯示一致的 focus 可視環，無障礙名稱與點擊行為與替換前相同

<!-- @trace
source: desktop-shadcn-controls
updated: 2026-07-08
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/settingsView.test.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/views/SettingsView.tsx
  - package-lock.json
  - packages/ui/package.json
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/components/ui/checkbox.tsx
  - packages/ui/src/components/ui/textarea.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 任務寫回非阻塞且序列化

任務寫回（單發勾選、批次設定、拖放排序、看板卡片排序）SHALL 於背景執行緒執行，SHALL NOT 阻塞視窗互動——寫回進行中使用者 SHALL 仍可捲動、點擊與觸發其他操作。同一專案的並發任務寫回 SHALL 依提交順序序列化落盤，SHALL NOT 遺失任何一次更新。完成路徑取得 git 身分 SHALL 每專案根至多 spawn 一次 git 並快取（app 存續期內重用），開工章 started_by 的內容 SHALL 與逐次取得時一致。樂觀更新生效後，更早發起而較晚到達的文件載入回應 SHALL NOT 覆蓋樂觀狀態。寫回過程 spawn 的 git 子進程 SHALL NOT 產生可見的主控台視窗（黑窗閃現）。

#### Scenario: 無主控台視窗閃爍

- **WHEN** 勾選任務觸發完成路徑的 git 呼叫（身分首抓或 touched 記錄）
- **THEN** 畫面上不出現任何主控台視窗閃現，git 呼叫結果與行為不變

#### Scenario: 慢寫回不凍結介面

- **WHEN** 勾選任務且後端寫回耗時數秒（如環境使 git 呼叫緩慢）
- **THEN** 視窗全程可操作（捲動、點擊、切換分頁皆有回應），無整窗凍結

#### Scenario: 勾選後立即取消不遺失

- **WHEN** 勾選一任務後在其寫回完成前取消勾選同一任務
- **THEN** 取消立即反映於畫面，兩次寫回依序落盤，最終 tasks.md 該任務為未勾選

#### Scenario: 並發寫回序列化

- **WHEN** 一筆慢寫回進行中又觸發另一任務的寫回
- **THEN** 兩筆寫回皆落盤且互不覆蓋，結果與依序執行一致

#### Scenario: git 身分快取重用

- **WHEN** 同一專案內連續勾選多個任務
- **THEN** git 身分僅首次（或啟動預熱時）取得，後續勾選不再逐次 spawn git，蓋章的 started_by 內容不變

#### Scenario: 舊載入回應不覆蓋樂觀狀態

- **WHEN** 樂觀勾選生效後，一筆更早發起的 tasks.md 載入回應才到達
- **THEN** 畫面維持樂觀勾選狀態不閃爍，寫回完成後由世代重載收斂至磁碟現況

<!-- @trace
source: desktop-toggle-freeze
updated: 2026-07-08
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/src/lib.rs
  - crates/speclink-core/src/util.rs
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
-->

---
### Requirement: 討論輪以卡片呈現

桌面 app 的討論過程呈現（討論抽屜的討論過程分頁、已封存討論檢視的討論過程分頁）SHALL 將符合 scaffold 格式（輪標題形如「### Round N — <mode> (<date>)」）的記錄逐輪呈現為獨立卡片：卡頭 SHALL 顯示輪次、mode 與日期；卡身 SHALL 將行首的 Focus／Position／Ruled out／Open 粗體前綴呈現為欄位標籤區塊，前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或輪結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；mode 值 SHALL 按來源字串透傳呈現（不設白名單）。任一輪標題不符 scaffold 格式時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準記錄逐輪成卡

- **WHEN** 開啟含四輪 scaffold 格式記錄的討論過程分頁
- **THEN** 呈現四張輪卡片，各卡頭顯示輪次、mode 與日期；卡身的 Focus 與 Position 以標籤區塊呈現，來源的粗體前綴原文不出現在渲染結果

##### Example: 輪標題解析

- **GIVEN** 來源輪標題「### Round 1 — assumptions (2026-07-08)」
- **WHEN** 開啟討論過程分頁
- **THEN** 該卡片卡頭顯示輪次 Round 1、mode assumptions、日期 2026-07-08

#### Scenario: 缺席欄位不渲染空標籤

- **WHEN** 某輪來源只有 Focus、Position 與 Open 行，無 Ruled out 行
- **THEN** 該卡片呈現 Focus、Position、Open 三個標籤區塊，無 Ruled out 標籤

#### Scenario: 欄位涵蓋後續多行內容

- **WHEN** 某輪的 Position 標籤行之後接數行列點、其後才是 Ruled out 標籤行
- **THEN** 該數行列點全數呈現於 Position 欄位區塊內，列表符號保留

#### Scenario: 非標準格式整篇退回

- **WHEN** 記錄的 Rounds 區段含不符 scaffold 格式的輪標題（手寫或 pre-scaffold 記錄）
- **THEN** 討論過程分頁整篇以單一 markdown 檢視呈現，無卡片、無錯誤訊息，來源檔案位元不變

<!-- @trace
source: drawer-document-readability
updated: 2026-07-08
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SectionedDoc.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 討論結論以欄位標籤呈現

桌面 app 的討論結論呈現（討論抽屜的結論分頁、已封存討論檢視的結論區）SHALL 將行首的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 粗體前綴呈現為欄位標籤區塊（標籤依語系呈現，zh-TW 為決定／理由／否決替代案／擱置／記錄去向／下一步），前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或結論結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；非六詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位。結論不含任何白名單欄位時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準結論欄位成標籤區塊

- **WHEN** 開啟含 scaffold 格式結論（Decision、Rationale、Capture to、Next 各佔一行起頭）的結論分頁
- **THEN** 各欄位以標籤區塊呈現，「**Decision**:」等粗體前綴原文不出現在渲染結果

#### Scenario: 結論缺席欄位不渲染空標籤

- **WHEN** 某結論來源只有 Decision 與 Rationale 行，無 Deferred 行
- **THEN** 結論分頁呈現決定與理由兩個標籤區塊，無擱置標籤

#### Scenario: 自由格式結論整篇退回

- **WHEN** 結論來源為手寫自由段落、不含任何白名單粗體前綴行
- **THEN** 結論分頁整篇以單一 markdown 檢視呈現，無標籤區塊、無錯誤訊息，來源檔案位元不變

<!-- @trace
source: drawer-document-readability
updated: 2026-07-08
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SectionedDoc.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: markdown 文件內容行寬有上限

桌面 app 經共用 markdown 渲染呈現的文件內容（變更抽屜的提案／設計／規格分頁、討論抽屜各分頁、規格抽屜、封存抽屜）SHALL 有固定行寬上限；抽屜寬度改變（含全螢幕）時內文行寬 SHALL NOT 隨之增長；容器寬於行寬上限時內容欄 SHALL 水平置中、留白均分兩側。同一抽屜分頁內的區段標籤、討論輪卡片與任務清單 SHALL 與內文同欄對齊。超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 全螢幕下行寬不增長且內容欄置中

- **WHEN** 變更抽屜切換至全螢幕（96vw）檢視提案分頁
- **THEN** 內文行寬維持固定上限，內容欄水平置中、左右留白均分，區段標籤與內文同欄對齊

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，抽屜版面不橫向溢出


<!-- @trace
source: specs-archive-pagination
updated: 2026-07-12
code:
  - packages/ui/src/__tests__/archivedDrawer.test.tsx
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/listPager.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/specDrawer.test.tsx
  - packages/ui/src/__tests__/specList.test.tsx
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/ListPager.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SpecDrawer.tsx
  - packages/ui/src/components/SpecList.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 規格分頁 delta 區段以色標呈現

變更抽屜規格分頁與已封存變更檢視的規格分頁 SHALL 將 delta spec 的區段標題（ADDED／MODIFIED／REMOVED／RENAMED Requirements）呈現為色標區段標頭，原始區段標題行 SHALL NOT 以標題文字直出；區段內的 requirement 與 scenario 內文 SHALL 照 prose 排版呈現。不含 delta 區段標題的規格文件 SHALL 整篇照常渲染。色標配色 SHALL 與 delta 計數徽章（DeltaBadges）一致。

#### Scenario: delta 區段呈現色標標頭

- **WHEN** 檢視含 ADDED 與 MODIFIED 區段的 delta spec
- **THEN** 呈現綠色「新增」與琥珀色「修改」區段標頭，原始「ADDED Requirements」「MODIFIED Requirements」標題文字不出現在渲染結果

##### Example: 四種 delta 區段的色標對應

| 來源區段標題 | 標頭文字 | 色系 |
| ------------ | -------- | ---- |
| ## ADDED Requirements | 新增 | 綠（emerald） |
| ## MODIFIED Requirements | 修改 | 琥珀（amber） |
| ## REMOVED Requirements | 移除 | 紅（red） |
| ## RENAMED Requirements | 更名 | 藍（sky） |

#### Scenario: 無 delta 標記的規格照常渲染

- **WHEN** 檢視不含任何 delta 區段標題的規格文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無色標標頭

<!-- @trace
source: drawer-document-readability
updated: 2026-07-08
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SectionedDoc.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 提案與設計章節以中文標籤呈現

變更抽屜與已封存檢視的提案／設計分頁 SHALL 將已知模板章節標題（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions）呈現為中文標籤區塊，英文模板標題 SHALL NOT 以標題文字直出；標籤款式 SHALL 為粗體大標題——計算字級 SHALL 大於內文基準字級（16px），且 SHALL 與討論側欄位標籤（輪的焦點／立場、結論的決定／理由）及規格分頁色標區段標頭同款式（色標標頭保留各 delta 色彩）。白名單以外的章節標題 SHALL 連同內文照 prose 排版呈現。整份文件無任何白名單章節時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 提案模板章節成中文標籤

- **WHEN** 檢視含 Why、What Changes、Non-Goals、Capabilities、Impact 章節的提案分頁
- **THEN** 呈現「為什麼」「變更內容」「非目標」「能力」「影響」標籤區塊，Why 等英文標題文字不出現在渲染結果

##### Example: 章節對照

| 來源章節標題 | 呈現標籤 |
| ------------ | -------- |
| ## Why | 為什麼 |
| ## What Changes | 變更內容 |
| ## Non-Goals | 非目標 |
| ## Context | 背景 |
| ## Decisions | 決策 |
| ## Risks / Trade-offs | 風險與取捨 |

#### Scenario: 標籤為大標題且字級大於內文

- **WHEN** 檢視提案分頁的章節標籤與討論抽屜結論分頁的欄位標籤
- **THEN** 兩者款式一致，皆為粗體且計算字級大於內文的 16px

#### Scenario: 白名單外章節照排

- **WHEN** 檢視設計分頁，其 Decisions 章節內含自訂決策標題（如 D1 起頭的三級標題）
- **THEN** 決策標題照 prose 標題樣式渲染，不被標籤化、不被翻譯

#### Scenario: 無白名單章節整篇退回

- **WHEN** 檢視手寫自由格式（無任何模板章節標題）的提案文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無標籤區塊、無錯誤訊息

<!-- @trace
source: drawer-section-labels
updated: 2026-07-08
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SectionedDoc.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 任務群組標題與章節標籤同款式

變更抽屜任務分頁的群組標題 SHALL 以標籤家族的次級款式呈現——粗體、計算字級與內文基準（16px）一致，與 Capabilities 次級標籤同款、與章節主標題同族但小一級；群組標題文字 SHALL 照來源呈現（不翻譯、不改寫）；任務勾選、拖曳排序與工具列行為 SHALL 不受款式變更影響。

#### Scenario: 群組標題款式一致

- **WHEN** 檢視含群組標題的任務分頁
- **THEN** 群組標題為粗體、計算字級為 16px（與任務文字同級、小於章節主標題），標題文字與來源一致

#### Scenario: 互動行為不變

- **WHEN** 在款式調整後的任務分頁勾選任務並拖曳排序
- **THEN** 勾選與排序行為與調整前一致，寫回結果正確

<!-- @trace
source: drawer-section-labels
updated: 2026-07-08
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/components.test.tsx
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/components/Markdown.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/SectionedDoc.tsx
  - packages/ui/src/components/TaskList.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 規格頁提供清單、搜尋與展開檢視

左側導覽的「規格」項 SHALL 進入規格頁，以卡片清單呈現全部正典 spec：每張卡 SHALL 含 spec 名稱、最後修改相對時間（自檔案系統 mtime 衍生，天級：今天／昨天／N 天前；mtime 不可得時該資訊缺席）、複製名稱鈕與展開／縮合控制。規格頁 SHALL 提供搜尋列，以大小寫不敏感的名稱子字串即時過濾清單。點卡片標題 SHALL 展開顯示該 spec 的正典 spec.md 全文（markdown 渲染），再點 SHALL 縮合；全文內容 SHALL 於首次展開時才載入。展開檢視的 spec.md 全文下方 SHALL 顯示一行來源變更 footer，列出該 spec 內所有 @trace 區塊的 source 變更名（去重、依文件首次出現順序）；spec.md 不含任何帶 source 的 @trace 時該 footer SHALL 缺席。此 footer SHALL 僅呈現、SHALL NOT 可點擊，且 SHALL NOT 顯示 @trace 的 updated 或 code。無 spec 的專案與搜尋無結果 SHALL 各顯示空狀態文案。規格頁 SHALL 為唯讀，SHALL NOT 提供任何規格寫入操作。

#### Scenario: 進入規格頁顯示卡片清單

- **WHEN** 於含多個正典 spec 的專案點左側導覽「規格」
- **THEN** 導覽項呈 active 樣式，主內容顯示全部 spec 卡片，各含名稱與最後修改相對時間

#### Scenario: 搜尋列名稱過濾

- **WHEN** 於搜尋列輸入部分 spec 名稱
- **THEN** 清單即時縮至名稱含該子字串（大小寫不敏感）的卡片；清空輸入後清單還原

##### Example: 過濾行為

| 既有 specs | 輸入 | 顯示 |
| ---------- | ---- | ---- |
| desktop-app、desktop-config、node-sdk | desktop | desktop-app、desktop-config |
| desktop-app、desktop-config、node-sdk | SDK | node-sdk |
| desktop-app、desktop-config、node-sdk | zzz | 無結果空狀態 |

#### Scenario: 展開卡片顯示正典全文

- **WHEN** 點一張縮合卡片的標題
- **THEN** 卡片展開顯示該 spec 的 spec.md 全文 markdown 渲染（首次展開先呈載入態），再點標題即縮合，其他已展開卡片不受影響

#### Scenario: 展開檢視顯示來源變更 footer

- **WHEN** 展開一張其 spec.md 含至少一個帶 source 的 @trace 的卡片
- **THEN** 全文下方顯示一行來源變更 footer，內容為該檔所有 @trace 的 source 去重、依首次出現順序排列，前置在地化標籤

##### Example: 來源去重與排序

| spec.md 內 @trace source 出現序 | footer 顯示 |
| ------------------------------- | ----------- |
| A、A、B | A、B |
| B、A、B | B、A |
| （無 @trace 或無 source） | 無 footer |

#### Scenario: 複製名稱

- **WHEN** 點卡片的複製名稱鈕
- **THEN** spec 名稱寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 無 spec 專案顯示空狀態

- **WHEN** 於無任何正典 spec 的專案進入規格頁
- **THEN** 顯示空狀態文案而非空白頁

#### Scenario: 外部變更後反映

- **WHEN** 規格頁開啟期間外部寫者修改某 spec 的 spec.md
- **THEN** 世代重載後清單的修改時間更新，已展開卡片的內容反映新全文


<!-- @trace
source: spec-source-footer
updated: 2026-07-09
code:
  - packages/ui/src/__tests__/specList.test.tsx
  - packages/ui/src/__tests__/trace.test.ts
  - packages/ui/src/components/SpecList.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/trace.ts
-->

---
### Requirement: 變更的來源討論多值呈現

變更連結多份討論時（meta 的 from_discussion 為逗號清單），變更卡 SHALL 維持單一討論徽章、以清單第一個（出身）討論為代表，徽章提示 SHALL 列出全部來源討論；變更詳情抽屜 SHALL 列出全部來源討論並可互跳至各討論的抽屜；同源變更清單 SHALL 以「雙方來源討論集合交集非空」判定收錄。單一來源討論的變更，其徽章、抽屜來源討論與同源變更清單的呈現 SHALL 與本變更前一致。

#### Scenario: 多來源徽章以出身討論為代表

- **WHEN** 看板呈現來源討論清單為兩份（出身在前）的變更卡
- **THEN** 卡片帶單一討論徽章、代表清單第一份（出身）討論，徽章提示列出全部兩份來源討論

#### Scenario: 詳情抽屜列出全部來源討論

- **WHEN** 開啟該變更的詳情抽屜
- **THEN** 來源討論區列出全部來源討論，點擊任一項開啟該討論的抽屜

#### Scenario: 同源以來源討論交集判定

- **WHEN** 變更 A 的來源討論清單為 d1, d2、變更 B 的來源討論清單僅含 d2
- **THEN** A 與 B 互為同源變更，出現在彼此詳情抽屜的同源變更清單

##### Example: 交集判定表

| 變更 A 來源 | 變更 B 來源 | 是否同源 |
| ----------- | ----------- | -------- |
| d1, d2      | d2          | 是       |
| d1          | d1          | 是       |
| d1, d2      | d3          | 否       |
| （無）      | d1          | 否       |

#### Scenario: 單一來源呈現不變

- **WHEN** 看板呈現僅一份來源討論的變更卡並開啟其詳情抽屜
- **THEN** 徽章、抽屜來源討論與同源變更清單的呈現與本變更前一致

<!-- @trace
source: rediscuss-promoted-change
updated: 2026-07-09
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/App.tsx
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/teststore.rs
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/siblings.test.ts
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/index.ts
  - packages/ui/src/siblings.ts
-->

---
### Requirement: 看板卡片浮現待重新反映徽章

desktop 看板的變更卡片 SHALL 於該變更 meta 的 restale_from 非空時，顯示一枚「待重新反映」徽章，提示該變更反映的討論已被重新結論、待 re-ingest。徽章的資料源 SHALL 為變更 meta 的 restale_from 欄位，經桌面看板查詢路徑的 Rust 變更序列化曝為資料欄、透過 tauriDataSource 傳至前端、由 packages/ui 的看板卡片元件渲染——全程僅讀既存 meta 欄位，SHALL NOT 於載入時掃描討論記錄。restale_from 為空或缺席時卡片 SHALL NOT 顯示該徽章。徽章 SHALL 與既有卡片視覺語言（主題化樣式）一致。此浮現不改變看板欄位派生規則（全完成＞有 started＞其餘）——徽章與欄位歸屬正交。

#### Scenario: 過期變更卡片顯示徽章

- **WHEN** 看板渲染一個 meta 帶非空 restale_from 的變更卡片
- **THEN** 該卡片顯示「待重新反映」徽章；徽章不影響卡片所在看板欄位

#### Scenario: 非過期變更卡片無徽章

- **WHEN** 看板渲染一個 meta 無 restale_from（或為空）的變更卡片
- **THEN** 該卡片不顯示「待重新反映」徽章，其餘呈現與本變更前一致

<!-- @trace
source: reconclude-restale
updated: 2026-07-09
code:
  - apps/desktop/core/src/query.rs
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/reconclude_restale.rs
  - crates/speclink-core/assets/skills/ingest.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 看板變更卡呈現建立者與關係提示

看板變更卡 SHALL 顯示建立者（createdBy）頭像——以建立者首字母圓標呈現，meta 無 created_by 時省略。變更卡的關係指示（來自討論、待重新反映）SHALL 於 hover 以主題化提示（shadcn Tooltip，取代原生 title）呈現對應資訊：來自討論指示於卡片來自討論時 SHALL 列出全部來源討論，待重新反映指示於卡片帶 restale 旗標時 SHALL 列出待重新反映的來源；無對應關係時該指示 SHALL 缺席。提示內容 SHALL 與原生 title 一致（改以主題化樣式呈現）。

#### Scenario: 變更卡顯示建立者頭像

- **WHEN** 看板呈現一個 meta 含 created_by 的變更卡
- **THEN** 卡片顯示該建立者的首字母圓標頭像

#### Scenario: 無建立者時省略頭像

- **WHEN** 變更卡的 meta 無 created_by
- **THEN** 卡片不顯示建立者頭像

#### Scenario: 關係指示以主題化 hover 提示呈現

- **WHEN** 使用者 hover 一個來自討論之變更卡的來自討論指示
- **THEN** 以主題化提示（shadcn Tooltip）列出全部來源討論，取代原生 title 呈現

<!-- @trace
source: desktop-card-identity
updated: 2026-07-09
code:
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/query.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-core/src/discuss.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/discussionDrawer.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/DiscussionDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: 拖曳封存落點以浮層呈現

拖曳看板卡片時的封存落點 SHALL 以浮層呈現：疊於看板欄列右緣上方、不參與欄列佈局——落點浮現與消失時各欄寬度 SHALL 維持不變。落點 SHALL 僅於拖曳變更卡時浮現；拖曳討論卡時 SHALL NOT 浮現（討論卡不可拖曳封存）。拖曳靠近落點時 SHALL 呈現可放開的視覺回饋。封存確認流程與拖排語意 SHALL 維持 board-card-order 規格所定行為不變。

#### Scenario: 拖曳變更卡時浮層浮現且欄寬不變

- **WHEN** 使用者開始拖曳任一變更卡
- **THEN** 封存落點以浮層疊於看板右緣浮現，各欄（含討論欄）寬度與拖曳前一致；放開或取消拖曳後浮層消失、欄寬仍不變

#### Scenario: 拖曳討論卡時落點不浮現

- **WHEN** 使用者開始拖曳討論卡
- **THEN** 封存落點不浮現，看板佈局無任何變動

#### Scenario: 拖至浮層落點放開觸發既有封存流程

- **WHEN** 使用者拖曳變更卡至浮層落點放開
- **THEN** 觸發既有封存確認流程，行為與未改版前一致

---
### Requirement: 看板卡片統一解剖學

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。複製鈕 SHALL 行內尾隨於標題最後一個字元後（標題折行時位於末行文字尾），meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。標題 SHALL NOT 截斷——完整顯示並自然折行。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情抽屜）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題折行時複製鈕仍緊跟末字元

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題折行完整顯示不截斷，複製鈕緊跟末行最後一個字元後且維持可點，右端 meta icons 不被擠出卡外

<!-- @trace
source: board-card-anatomy
updated: 2026-07-11
code:
  - apps/desktop/core/src/query.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/DiscussionColumn.tsx
-->

---
### Requirement: 已封存項目以抽屜檢視

已封存頁的變更列與討論列 SHALL 以卡片清單呈現，點擊卡片 SHALL 開啟唯讀抽屜，SHALL NOT 提供行內展開。封存變更抽屜 SHALL 至少含提案、設計、任務、規格分頁，內容來自封存目錄的實體文件；任務分頁的核取方塊不可點擊且無批次工具列；有來源討論的封存變更 SHALL 於標題下方顯示可點的來源討論標記，點擊 SHALL 於同一抽屜切換至該討論的唯讀檢視（記錄以 live 優先、封存後備載入），無來源討論時該標記 SHALL 缺席。封存討論抽屜 SHALL 呈現「背景」「討論過程」「結論」區段。兩抽屜 SHALL 為唯讀——SHALL NOT 提供任務勾選、動詞執行或任何寫入操作；寬度與全螢幕切換 SHALL 與變更詳情抽屜一致；開啟期間內容 SHALL 隨外部檔案變更反映。所請求的文件不存在時對應分頁或區段 SHALL 顯示空狀態而非錯誤。

#### Scenario: 點擊封存變更卡開啟四分頁抽屜

- **WHEN** 使用者於已封存頁點擊一個含完整 artifacts 的封存變更卡
- **THEN** 開啟唯讀抽屜顯示提案／設計／任務／規格分頁，各分頁呈現封存目錄內對應文件的內容，任務分頁的核取方塊不可點擊，清單卡片本身無行內展開

#### Scenario: 點擊封存討論卡開啟唯讀抽屜

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論卡
- **THEN** 開啟唯讀抽屜顯示背景、討論過程、結論區段，無轉為變更、封存或任何寫入按鈕

#### Scenario: 缺件文件顯示空狀態

- **WHEN** 使用者開啟一個無 design.md 的封存變更抽屜並切至設計分頁
- **THEN** 分頁顯示空狀態文字，無錯誤彈窗，其餘分頁照常可用

#### Scenario: 自封存變更抽屜跳轉來源討論

- **WHEN** 使用者開啟一個帶來源討論的封存變更抽屜，點擊標題下方的來源討論標記
- **THEN** 同一抽屜切換為該討論的唯讀檢視（背景／討論過程／結論），記錄以 live 優先、封存後備載入；無來源討論的封存變更抽屜不顯示此標記

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
### Requirement: 規格與封存卡片收合資訊

規格卡 SHALL 於標題文字後緊跟複製名稱鈕，並顯示需求數、溯源變更數與相對修改時間；Purpose 摘要 SHALL 取正典 Purpose 區段首個非空行一行截斷顯示，當 Purpose 為封存流程產生的佔位文字時 SHALL 改顯「Purpose 待補」警示樣式而非佔位原文。封存變更卡 SHALL 顯示日期、標題後緊跟複製鈕、任務數徽章（未全完成 SHALL 以警示樣式呈現，與全完成可辨；無 tasks.md 者不顯示徽章）、觸及規格數、建立者標記（hover 顯示全名）與來源討論標記（無來源討論時缺席）。封存討論卡 SHALL 顯示日期、topic、複製 slug 鈕、輪數與衍生變更數。三種卡片的計數 meta（需求數、溯源變更數、觸及規格數、衍生變更數）SHALL 以一致的「icon＋數字」樣式呈現——SHALL NOT 混用 pill 底色或無 icon 的圓圈數字；任務數徽章 SHALL 維持 pill 樣式與配色分級（狀態語意例外）。上述資訊 SHALL 於收合狀態（未開啟任何抽屜）即可見，資料 SHALL 由清單載入一次帶出，SHALL NOT 逐卡讀取文件全文。

#### Scenario: 規格卡收合資訊

- **WHEN** 規格頁載入一份含 7 條 Requirement、Purpose 已填寫、溯源自 3 個變更的規格
- **THEN** 該卡於收合狀態顯示需求數 7、溯源變更數 3、Purpose 首行摘要與相對修改時間，全程未讀取其他規格全文

#### Scenario: Purpose 佔位顯示待補提示

- **WHEN** 某規格的 Purpose 區段仍為封存流程產生的佔位文字（精確偵測字串由實作與封存產生器以同一單元測試釘住）
- **THEN** 該規格卡以警示樣式顯示「Purpose 待補」，不顯示佔位原文

#### Scenario: 封存變更卡任務徽章配色分級

- **WHEN** 已封存頁含一筆 20/21 未全完成與一筆 48/48 全完成的封存變更
- **THEN** 前者任務徽章以警示樣式顯示 20/21，後者以一般樣式顯示 48/48，收合狀態即可一眼區辨

#### Scenario: 封存討論卡顯示衍生變更數與複製 slug

- **WHEN** 已封存頁討論節含一筆轉出 2 個變更的封存討論
- **THEN** 該卡顯示衍生變更數 2 與複製 slug 鈕，點擊複製鈕將 slug 寫入剪貼簿且不開啟抽屜

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
### Requirement: 清單最新在前與換頁瀏覽

規格頁與已封存頁的清單 SHALL 以最新在前排序：封存變更依封存日期新→舊（同日由封存目錄名字典序降冪涵蓋）、封存討論依建立日期新→舊（同日以 slug 字母升冪決勝）、規格依最後修改時間新→舊（修改時間不可得者一律排最後、以名稱字母升冪決勝）。上述清單 SHALL 以每頁 20 筆換頁瀏覽。清單頁版面 SHALL 填滿視窗高度且頁面本身不縱向捲動：搜尋框（與已封存頁的子頁籤列）固定於頂部、卡片清單於內部容器縱向捲動。搜尋過濾後超過 20 筆時，換頁控制列——上一頁鈕、目前頁與總頁數、下一頁鈕——SHALL 常駐固定於頁面底部，不隨清單內容捲動移出視窗（不捲動即可換頁）；第 1 頁時上一頁鈕 SHALL 不可用、末頁時下一頁鈕 SHALL 不可用；過濾後 20 筆以內換頁控制列 SHALL 缺席。搜尋字串變更 SHALL 使清單回到第 1 頁；換頁後內部捲動容器 SHALL 捲回頂部；清單縮短使目前頁碼越界時 SHALL 鉗制至末頁而非顯示空頁。已封存頁「變更」「討論」兩子頁籤 SHALL 維持各自獨立的換頁控制列與頁碼。排序與換頁 SHALL 僅屬前端呈現，清單載入行為與清單資料欄位 SHALL NOT 因此改變。

#### Scenario: 封存變更最新在前

- **WHEN** 已封存頁含封存於 2026-07-04、2026-07-11、2026-07-08 的三筆封存變更
- **THEN** 「變更」子頁籤清單依 2026-07-11、2026-07-08、2026-07-04 順序呈現

#### Scenario: 規格修改時間缺席排最後

- **WHEN** 規格頁含修改時間可得的規格 alpha（3 天前）、beta（今天）與修改時間不可得的 zeta、delta
- **THEN** 清單順序為 beta、alpha、delta、zeta——可得者新→舊在前，不可得者殿後並依名稱字母序排列

#### Scenario: 超過 20 筆顯示換頁控制列

- **WHEN** 已封存頁「變更」子頁籤過濾後有 45 筆
- **THEN** 清單顯示第 1 頁的 20 筆，換頁控制列常駐於頁面底部（上一頁不可用、頁數顯示第 1／3 頁）；點下一頁顯示第 21–40 筆並使內部捲動容器捲回頂部

#### Scenario: 換頁控制列不捲動即可觸及

- **WHEN** 已封存頁「變更」子頁籤過濾後超過 20 筆，視窗高度僅能顯示部分卡片，且使用者尚未捲動
- **THEN** 換頁控制列已於頁面底部可見且可點擊；捲動卡片清單（內部容器）時，搜尋框、子頁籤列與換頁控制列維持固定位置，不隨內容移出視窗

#### Scenario: 20 筆以內換頁控制列缺席

- **WHEN** 規格頁清單過濾後僅 13 筆
- **THEN** 全部 13 筆單頁呈現，無換頁控制列

#### Scenario: 搜尋字串變更回到第 1 頁

- **WHEN** 使用者於「變更」子頁籤第 3 頁修改搜尋字串
- **THEN** 清單回到第 1 頁顯示新過濾結果的最前 20 筆


<!-- @trace
source: archived-page-fill-height
updated: 2026-07-12
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - packages/ui/src/__tests__/archivedList.test.tsx
  - packages/ui/src/__tests__/specList.test.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/SpecList.tsx
-->

---
### Requirement: detail 抽屜互斥

桌面 app SHALL 保證同時僅一個 detail 抽屜（變更詳情、規格、封存、討論）開啟：任一 detail 抽屜開啟中，自任何入口（看板卡片、規格清單、封存頁、系統匣選單與面板、抽屜內跳轉）開啟另一種 detail 抽屜時，先開者 SHALL 關閉、由後開者取代顯示，SHALL NOT 疊加顯示、SHALL NOT 拒開。此互斥 SHALL 由狀態層的開啟動作保證——每個開啟動作於設定自身選定狀態時同時清除其他抽屜的選定狀態；SHALL NOT 依賴各呼叫端入口自行先關再開。

#### Scenario: 討論抽屜開啟中自系統匣開啟變更詳情

- **WHEN** 討論抽屜開啟中，使用者自系統匣開啟某 change 的詳情抽屜
- **THEN** 討論抽屜關閉，僅變更詳情抽屜可見

#### Scenario: 任兩種 detail 抽屜交錯開啟

- **WHEN** 任一 detail 抽屜開啟中，使用者自任何入口開啟另一種 detail 抽屜
- **THEN** 先開者關閉、後開者顯示，同時僅一個 detail 抽屜可見

##### Example: 四種抽屜互斥狀態轉移

| 開啟中的抽屜 | 開啟動作         | 結果                       |
| ------------ | ---------------- | -------------------------- |
| 討論         | 開啟變更詳情     | 討論關閉，僅變更詳情可見   |
| 變更詳情     | 開啟規格         | 變更詳情關閉，僅規格可見   |
| 規格         | 開啟封存         | 規格關閉，僅封存可見       |
| 封存         | 開啟討論         | 封存關閉，僅討論可見       |

<!-- @trace
source: drawer-exclusivity
updated: 2026-07-17
code:
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/store.ts
-->
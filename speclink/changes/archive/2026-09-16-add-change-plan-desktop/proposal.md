## Why

第一刀（add-change-plan-engine）把執行順序的計算放進引擎，但桌面看板與系統匣仍只照 `board_rank` 與修改時間排卡片，使用者從畫面上看不出誰該先做、誰可以並行，也沒有地方查看或調整一個 change 的前置。本變更是討論 change-execution-order 的第二刀（共三刀）：讓 local 模式的看板三欄、系統匣與詳情面板都照引擎算出的順序呈現，卡片標出波次與阻擋，拖排不得跨越宣告依賴。目標使用者是用桌面 app 看進度、手動排優先序的開發者與 PM，對應 apply 前的挑選階段與封存前的排隊階段。

## What Changes

- 桌面變更清單 payload（`list_changes` command）改以引擎 plan 的配置順序回傳，並在每個 change 項加 `wave`（整數）、`blockedBy`（字串陣列）、`dependsOn`（字串陣列）、`overlaps`（陣列，每項 `change` 與 `capabilities`）四個呈現層欄位，頂層加 `planError`（依賴成環時為訊息字串，否則 null）。CLI `list --json` 不變。
- 看板三個生命週期欄（提案中／進行中／已就緒）的顯示順序 = plan 配置順序；缺 rank 的卡改排在有 rank 的卡之後、依建立日期升冪（原本置欄頂、回退修改時間）。**BREAKING**（看板顯示序）：全員缺 rank 的欄不再與引入排序能力前一致，改為建立日期升冪。
- 變更卡於進度列最左新增波次章（圓圈數字，同波同號＝可並行；tooltip 帶前置清單）；被擋的卡整卡變淡（與系統匣面板同一語意），標題列不因排程資訊增加任何章；remote 模式缺欄位時章缺席、不變淡。
- 拖排受宣告依賴約束：拖動時不合法落點（會把卡放到宣告前置之前、或宣告依賴它的卡之後）灰化且放下無效，判定以同欄全序計算（搜尋過濾不影響）；桌面 core 寫 rank 前呼叫引擎的 rank 移動檢查，被拒即以單行錯誤呈現並刷新回磁碟現況；欄內 rank 序與顯示序不一致時先依顯示序整欄重派再套移動。delta 重疊的夥伴可以互換先後。
- 依賴成環時看板不崩：卡片照回退順序顯示、無波次章，看板頂部顯示一行「依賴成環：a → b → a」的提示；清單項仍帶 `dependsOn`，排程分頁只剩前置段（可移除）讓使用者解環。
- 變更詳情面板新增「排程」分頁：波次與同波夥伴、前置清單（可加可刪，寫回 `depends_on`）、重疊清單（唯讀）、阻擋清單。remote 模式（第三刀前）分頁唯讀、不顯示編輯控制項。
- 系統匣：macOS 面板的變更列列首加同一波次數字、被擋的列變淡；原生選單的列標籤前綴波次數字；順序與看板同源。
- 新增 Tauri command `set_change_depends` 與資料源方法 `setDepends`，單行委派到桌面 core，再呼叫第一刀的引擎寫入函式；桌面 core 於新增後以看板同源視圖複檢成環（worktree 映射下引擎只看副本），成環即撤銷並回錯。
- 桌面 core 的階段判定改呼叫第一刀新增的引擎 `stage` 函式，不再自持一份規則。
- 相容性影響：桌面 payload 只加欄位不改既有欄位；CLI 輸出不變；看板顯示序刻意變更如上。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `board-card-order`: 「看板卡片順序以 board_rank 欄位為真相」改為 local 順序 = plan 配置順序（rank 為輸入、缺 rank 置後依建立日期）；「欄內拖排以中點 rank 單檔寫回」加入寫回前的依賴檢查與拒絕行為。
- `desktop-app`: 新增看板卡片的波次與阻擋標示（波次章在進度列、被擋整卡變淡）、拖排時不合法落點灰化、依賴成環的看板提示、詳情抽屜的排程分頁四條需求。
- `tray-status-menu`: 新增面板與原生選單變更列的波次標示需求。
- `client-protocol`: 新增變更清單的排程欄位需求（desktop 協定 local payload；remote 摘要留待第三刀）。

## Impact

- Affected specs: `board-card-order`、`desktop-app`、`tray-status-menu`、`client-protocol`（修改）
- Affected code:
  - New: packages/ui/src/__tests__/planBadge.test.tsx、packages/ui/src/__tests__/planTab.test.tsx、apps/desktop/src/__tests__/planDataSource.test.ts
  - Modified: apps/desktop/core/src/query.rs（清單改以 plan 順序與四欄位、planError）、apps/desktop/core/src/manage.rs（reorder 前的依賴檢查與反序整欄重派、set_depends_at 與 overlay 成環複檢、change_stage 改呼叫引擎）、apps/desktop/core/src/testfixture.rs（共用的 board_names）、apps/desktop/src-tauri/src/lib.rs（set_change_depends command 與註冊）、apps/desktop/src-tauri/src/remote.rs（RemoteCapabilities.set_depends 旗標）、apps/desktop/src/adapter/tauriDataSource.ts、apps/desktop/src/adapter/remoteDataSource.ts（setDepends 不可用）、apps/desktop/src/session.ts（capability 旗標）、apps/desktop/src/store.ts、apps/desktop/src/App.tsx（planError 提示接線）、apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/tray.ts、packages/ui/src/adapter.ts（ChangeItem 四欄位與 setDepends）、packages/ui/src/index.ts（新入口的匯出）、packages/ui/src/stage.ts（波次與阻擋讀取入口、文字組裝）、packages/ui/src/boardDnd.ts（不合法落點集合）、packages/ui/src/components/KanbanBoard.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx（排程分頁）、packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts、packages/ui/src/__tests__/kanban.test.tsx、packages/ui/src/__tests__/stage.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/tauriDataSource.test.ts、apps/desktop/src/__tests__/remoteDataSource.test.ts、apps/desktop/src/__tests__/helpers/remoteFixtures.ts（setDepends 夾具與 capability 旗標）
  - Removed: 無
- 前置相依：add-change-plan-engine 必須先落地（本變更呼叫其 `plan::compute`、`plan::check_rank_move`、`plan::set_depends`、`model::stage`）。
- 不動的部分：crates/ 下任何 crate；remote 資料源的排序與編輯（第三刀）。

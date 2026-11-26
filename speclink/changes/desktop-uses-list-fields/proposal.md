## Why

變更 change-and-discussion-list-fields 讓引擎與 HTTP 回應帶出階段、建立者的識別、就緒日與討論的修改時間。Speclink Desktop 是官方的畫面，卻還沒用上它們：看板的階段由 Desktop 自己再算一次（同一條規則在引擎以外還有兩份）；建立者的篩選用名稱全等比對，改過名稱的人會被拆成兩個選項、同名的兩個人會被合成一個；卡片與詳情頁看不到「什麼時候就緒」與「最後什麼時候動過」。討論 embedded-host-needs-10 的結論決定 Desktop 這一輪一併用上這四項。

目標使用者是在 Desktop 的看板與詳情頁追進度的開發者、PM 與 PO，對應 apply 之後到封存之前的追蹤。

## What Changes

- 就緒日：變更詳情頁左欄的出身行，在開工日之後接「就緒 YYYY-MM-DD」。只在變更有就緒日時出現（目前全部任務都勾完，而且引擎記下了時間）。
- 最後更新：看板的變更卡與討論卡在 meta 列顯示最後更新的相對時間（今天、昨天、N 天前），取清單的 `modifiedAt`。沒有值時不出現。卡片的順序不變，仍然是拖排的順序。
- 階段：看板、系統匣面板與詳情頁判斷階段時，清單項帶有 `stage` 就用它，沒有才照既有的規則自己算（連到舊版 Server 時）。畫面不變。
- 建立者篩選：看板篩選面板的「建立者」選單改成用識別認人。卡片帶有 `createdById` 時依識別分組，名稱只當標籤；沒有識別的卡片照舊用名稱。同一個名稱只對應到一個識別時，沒有識別的舊卡片併進那個人的選項。
- 資料：Desktop 本機專案的變更清單帶上 `stage` 與 `createdById`，單一變更的資料帶上 `readyAt` 與 `createdById`；遠端專案的同名欄位取自 Server 的回應。討論清單的 `modifiedAt` 與 `createdById` 由引擎的輸出帶出。
- 不新增也不變更任何 CLI 指令、旗標、`--json` 欄位、HTTP 路由與設定欄位；不改任何技能檔。

### 相容性影響

- 本機專案沒有建立者的識別，所以建立者篩選的行為和今天相同。
- 連到舊版 Server 的遠端專案：清單沒有 `stage`、`createdById`，單一變更沒有 `readyAt`；階段照既有規則自己算，篩選照名稱，就緒日不出現。
- 卡片的 meta 列多一段文字；卡片的高度與其他段落不變。
- 本機專案的最後更新取自檔案時間，checkout 或複製也會改到它，所以顯示的時間有機會比實際的最後編輯新。
- CLI 的輸出與 HTTP 回應都不受這個變更影響。

### 三種形態

- 本機 repo：可用。階段、最後更新與就緒日都有；建立者篩選照名稱（沒有識別）。
- 連到遠端的 checkout：可用，要等 Server 升級到含 change-and-discussion-list-fields 的版本；之前退回既有行為。
- 沒有本機規格檔的內嵌宿主：不適用。這個變更只改 Speclink Desktop 的畫面與它的資料組裝；宿主自己的畫面直接讀動詞的輸出。Desktop 連到內嵌宿主的 HTTP 入口時，行為和「連到遠端的 checkout」相同。

## Non-Goals (optional)

見 design 的 Goals / Non-Goals。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 新增三條需求（出身行顯示就緒日、看板卡片顯示最後更新、看板欄位以清單帶的階段為準），並修改「看板搜尋過濾卡片」的建立者選單。
- `client-protocol`: 新增一條需求，規定桌面的變更清單、單一變更與討論清單帶階段、建立者識別、就緒日與修改時間。

查過的相關正式規格：desktop-app（看板欄位由生命週期標記驅動、看板卡片統一解剖學、看板搜尋過濾卡片、變更詳情頁的標頭與狀態卡、討論詳情頁）、client-protocol（變更清單的建立者與來源討論欄位、變更清單的排程欄位）、board-card-order（卡片順序以 board_rank 為真相）、tray-status-menu、ui-copy-vocabulary。board-card-order 與 tray-status-menu 的行為不變。

## Impact

- Affected specs: desktop-app、client-protocol
- Affected crates and apps: apps/desktop（desktop-core 與 Tauri 殼）、packages/ui
- 前置變更：change-and-discussion-list-fields（引擎的階段函式、metadata 的新欄位與 HTTP 回應的新欄位都由它提供）
- Affected code:
  - Modified:
    - packages/ui/src/adapter.ts
    - packages/ui/src/stage.ts
    - packages/ui/src/search.ts
    - packages/ui/src/i18n.tsx
    - packages/ui/src/components/KanbanBoard.tsx
    - packages/ui/src/components/ChangeCard.tsx
    - packages/ui/src/components/DiscussionColumn.tsx
    - packages/ui/src/components/ChangeDetailPage.tsx
    - packages/ui/src/__tests__/stage.test.ts
    - packages/ui/src/__tests__/search.test.ts
    - packages/ui/src/__tests__/kanban.test.tsx
    - packages/ui/src/__tests__/discussionColumn.test.tsx
    - packages/ui/src/__tests__/changeDetailPage.test.tsx
    - apps/desktop/core/src/query.rs
    - apps/desktop/core/src/manage.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src-tauri/tests/it/remote_data.rs
  - New: (none)
  - Removed: (none)

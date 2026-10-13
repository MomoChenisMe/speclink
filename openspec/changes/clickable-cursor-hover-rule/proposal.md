## Summary

統一桌面 app 與網頁後台可點元素的游標與 hover 回饋：桌面版一律維持箭頭、以看得出的 hover 底色表示可點（參考 Codex app），網頁版一律手指，停用與唯讀為箭頭且無 hover；游標由共用樣式的單一設定集中決定，元件不再各自寫死手指游標。

## Motivation

使用者（透過桌面 app 與網頁後台檢視變更、規格、討論的開發者與 PO／PM）在 desktop-discussion-doc-views 的手動驗收（[M] 6.2）中發現：同樣可點的元素，有的滑過會變手指、有的不會，有的連 hover 底色都沒有。盤點結果：

- 游標沒有規則，由每個元件自己決定：清單列、任務列、看板卡片、四種收合卡頭、部分名稱籤各自寫了 `cursor-pointer`；`Button` 元件全部用法（約 105 處）、專案欄、文件樹、選單項沿用 Tailwind v4 預設的箭頭；另有 8 處長得像籤或連結的可點元素沒有手指。漏加是靠「每個元件記得」這種做法的必然結果。
- 停用的元素一半用箭頭、一半用禁止游標（`cursor-not-allowed`），開啟專案畫面的選擇卡停用後仍有 hover 底色。

參考 Codex app（`com.openai.codex`，Electron）打包的樣式：可點元素的游標一律寫成 `cursor: var(--cursor-interaction, pointer)`，桌面版在根層把這個變數設成 `default`，所以同一份程式在網頁是手指、在桌面全部是箭頭，可點性由 hover 底色表示。使用者選擇照這個做法。

前提：hover 底色已於 2026-10-12 驗收時統一為前景色 5%（輪次卡頭、設計決策卡頭、任務列、清單列、規格頁「+N」籤、最近開啟列、遠端復原的技術細節鈕，共 7 處），隨 desktop-discussion-doc-views 的驗收修正提交；本變更建立在「每個可點元素都有看得出的 hover」之上，只處理游標與停用態。

## Proposed Solution

- 共用樣式新增一個游標變數，預設值為手指；桌面 app 的根樣式（主視窗與系統匣面板共用）把它設成箭頭。
- 共用樣式加一條全域規則：可用的原生按鈕與 `role="button"` 元素的游標取這個變數。新寫的元件不用記得加游標。
- 元件與元件庫原語不再寫死 `cursor-pointer`：現有 15 處元件（含驗收時新加的遠端復原技術細節鈕）與開關原語改取同一個變數；選單項與下拉選項原本寫死箭頭，也改取變數（網頁是手指、桌面是箭頭）。網頁後台自有的 3 處同樣改取變數。
- Markdown 正文的超連結維持瀏覽器預設的手指（不受全域規則影響）。
- 停用與唯讀：游標一律箭頭（元件庫原語的 `cursor-not-allowed` 改成箭頭），且不給 hover 底色；開啟專案畫面停用的選擇卡與最近開啟列補上這個條件。
- 新增守門測試：掃描 packages/ui、桌面 app、網頁後台的元件原始碼，出現寫死的 `cursor-pointer` 或 `cursor-not-allowed` 即失敗；並檢查共用樣式的變數預設值、全域規則與桌面版覆寫都在。
- 規格：desktop-app 新增「可點元素的游標與 hover 回饋」需求；server-web-console 新增「網頁後台可點元素顯示手指游標」需求；tray-status-menu 修改「面板交接主視窗後滑鼠互動完整」，把「游標樣式隨元素改變」改成依新規則可觀察的說法。

相容性影響：純視覺。桌面版原本有手指的 15 處元件（含上次 desktop-spec-manual-pages 為需求收合列與工單輪卡頭加的手指）改為箭頭，hover 底色保留；網頁後台的按鈕、分頁、選單項由箭頭改為手指。不涉及 CLI、`--json` 輸出、設定欄位與技能。

## Non-Goals

- 不改 hover 底色的深淺（已統一為前景色 5%，見 Motivation 的前提）。
- 不改拖曳把手的抓取游標、文字輸入的文字游標、調整尺寸的游標。
- 不引入使用者可切換的游標偏好設定（Codex 也沒有；網頁與桌面由宿主決定）。
- 不改系統匣原生選單（非 webview）的游標。

## Alternatives Considered

- 能點的都給手指（網頁慣例，一條全域規則）——簡單，但桌面版的按鈕、分頁、側欄也變手指，不像 macOS 原生 app；使用者參考 Codex 後選擇桌面維持箭頭。
- 只有內容裡的可點物（列、籤、收合卡頭）給手指、按鈕維持箭頭——要逐個元件判斷，仍靠記得，且桌面上同一頁會手指與箭頭混用；不採用。
- 元件各自加 `cursor-pointer` 並以白名單守門——規則分散在百處以上的呼叫端，不採用。

## Capabilities

### New Capabilities

（none）

### Modified Capabilities

- `desktop-app`: 新增「可點元素的游標與 hover 回饋」需求（桌面箭頭、hover 表示可點、Markdown 連結手指、停用唯讀箭頭無 hover、游標集中設定與守門）。
- `server-web-console`: 新增「網頁後台可點元素顯示手指游標」需求（共用設定的預設值、停用唯讀箭頭無 hover）。
- `tray-status-menu`: 修改「面板交接主視窗後滑鼠互動完整」，游標的可觀察條件改依 desktop-app 的游標規則。

## Impact

- Affected specs: desktop-app（ADDED 1）、server-web-console（ADDED 1）、tray-status-menu（MODIFIED 1）
- Affected apps: packages/ui（共用樣式、元件庫原語、元件）、apps/desktop（根樣式、開啟專案畫面）、apps/server-web（清單元件）
- Affected code:
  - New: packages/ui/src/__tests__/cursorRule.test.ts
  - Modified: packages/ui/src/theme.css、apps/desktop/src/index.css、packages/ui/src/components/ui/switch.tsx、packages/ui/src/components/ui/dropdown-menu.tsx、packages/ui/src/components/ui/select.tsx、packages/ui/src/components/ui/checkbox.tsx、packages/ui/src/components/ui/input.tsx、packages/ui/src/components/ui/textarea.tsx、packages/ui/src/components/ui/label.tsx、packages/ui/src/components/DesignDoc.tsx、packages/ui/src/components/ManualPage.tsx、packages/ui/src/components/ContextDoc.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/ListRow.tsx、packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/RoundsDoc.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/SpecDetailPage.tsx、packages/ui/src/components/TicketView.tsx、packages/ui/src/components/RequirementBlock.tsx、apps/desktop/src/components/RemoteWorkspaceRecovery.tsx、apps/desktop/src/components/WorkspaceChooser.tsx、apps/desktop/src/components/RecentList.tsx、apps/server-web/src/components/DataList.tsx、apps/server-web/src/pages/admin/RegistryPage.tsx、packages/ui/src/__tests__/listRow.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/deltaSpecView.test.tsx、packages/ui/src/__tests__/specDetailPage.test.tsx、packages/ui/src/__tests__/ticketView.test.tsx、packages/ui/src/__tests__/roundsDoc.test.tsx、packages/ui/src/__tests__/designDoc.test.tsx、packages/ui/src/__tests__/ui.test.tsx、apps/desktop/src/__tests__/workspaceChooser.test.tsx、apps/desktop/src/__tests__/recentList.test.tsx
  - Removed: （none）

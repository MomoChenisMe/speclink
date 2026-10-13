## 1. 未轉出判定的單一來源

- [ ] 1.1 先寫紅燈（design D2；spec「討論結論以欄位標籤呈現」Example「刀的轉出判定」）：`packages/ui/src/__tests__/discussionModel.test.ts` 新增群組「cutPromotion」四題——變更名 `a`、`b` 全在已轉出集合回 `"promoted"`；只有 `a` 在回 `"partial"`；`a` 不在回 `"none"`；`names` 為空陣列回 `"unknown"`（`isPromoted` 以 `Set.has` 傳入）。驗證：`npm test -w packages/ui -- discussionModel` 以「cutPromotion 未匯出」失敗，既有題目照綠。 <!-- speclink-task:tsk_01M4SQWTHRHDT6TS4NFMB1YDG0 -->
- [ ] 1.2 讓 1.1 轉綠：`packages/ui/src/discussionModel.ts` 匯出純函式 `cutPromotion(cut: CutCard, isPromoted: (name: string) => boolean)`，依 design D2 回 `"promoted" | "partial" | "none" | "unknown"`，不讀任何其他狀態。驗證：`npm test -w packages/ui -- discussionModel` 全綠。 <!-- speclink-task:tsk_01M4SQWTHRCFH2X8RDBWRHEG2A -->

## 2. 刀卡的未轉出標示

- [ ] 2.1 先寫紅燈（design D3；spec「討論結論以欄位標籤呈現」的「分刀卡與變更籤」）：`packages/ui/src/__tests__/statusPill.test.tsx` 的 `TONE_CLASSES` 加 `["manual", ["bg-status-manual/12", "text-status-manual"]]`；`packages/ui/src/__tests__/conclusionDoc.test.tsx` 的「三張分刀卡的識別籤與範圍句、前兩張變更籤帶階段籤且點擊呼叫 stageOf 回的 onOpen、第三張不可點」補斷言——第三張卡帶 `data-cut-todo`、className 含 `border-dashed` 與 `border-status-manual/40`、卡頭含文字「尚未轉出」，前兩張卡無 `data-cut-todo` 且不含「尚未轉出」；新增兩題「部分轉出：卡無 data-cut-todo，只有未轉出的變更名籤之後有『尚未轉出』」「刀列沒有變更名籤時卡無 data-cut-todo、無『尚未轉出』」。驗證：`npm test -w packages/ui -- statusPill conclusionDoc` 中新增斷言失敗、其餘題目照綠。 <!-- speclink-task:tsk_01M4SQWTHR3T0EJ4CKNFCTJRX2 -->
- [ ] 2.2 讓 2.1 轉綠：`packages/ui/src/components/ui/status-pill.tsx` 的 `StatusPillTone` 與色調表加 `manual`；`packages/ui/src/components/ConclusionDoc.tsx` 的 `CutCardView` 以 `cutPromotion(cut, (n) => stageOf?.(n) != null)` 判定，依 design D3 畫整刀未轉出（虛線＋`SEMANTIC_SURFACE.manual`、`data-cut-todo`、卡頭 `StatusPill tone="manual"`）與部分轉出（未轉出變更名籤後接同款籤）；`packages/ui/src/i18n.tsx` 新增 `conclusion.notPromoted`（tw「尚未轉出」、en「Not promoted」）。驗證：`npm test -w packages/ui -- statusPill conclusionDoc` 全綠。 <!-- speclink-task:tsk_01M4SQWTHRNBW33FZ3GV2KSGEV -->

## 3. 結論提示框與「前往」

- [ ] 3.1 先寫紅燈（design D4；spec「討論結論以欄位標籤呈現」的「未轉出刀的提示框與前往」「沒有整刀未轉出的刀時無提示框」「已封存討論頁不出未轉出提示框」）： <!-- speclink-task:tsk_01M4SQWTHRF72MZXCFE9VRZJT7 -->
  - `packages/ui/src/__tests__/discussionDetailPage.test.tsx` 的「數字磚、出處行與提示框」群組新增三題：「結論有整刀未轉出的刀時有橄欖色提示框」——Decision 含 cut 1 `add-team-plugin-library` 與 cut 2 `remove-builtin-wad-work`、`rows` 只含 add-team-plugin-library、names 的 `self` 為 `wad-plugins-merge-or-split`，斷言 `[data-doc-callout]` 標題為「1 刀尚未轉出」、className 含 `status-manual`、恰一列 `[data-unpromoted-cut]` 含「cut 2」、指令籤文字為「/speclink-propose --from-discussion wad-plugins-merge-or-split」；「全部刀已轉出或只部分轉出時無提示框」；「status 為 archived 時無提示框」。既有「結論文件的數字磚與出處行」的 `rows` 改為含 chg-1、chg-2、chg-3 三列，維持無提示框。
  - `packages/ui/src/__tests__/conclusionDoc.test.tsx` 新增「jump 請求把該刀卡捲到畫面中央並呼叫 onJumped」（spy `Element.prototype.scrollIntoView`，斷言呼叫在 `[data-cut="cut 2"]` 上且參數含 `block: "center"`）。
  - `packages/ui/src/__tests__/discussionDetailPage.test.tsx` 新增頁面題「點提示框的『前往』後 cut 2 的刀卡被捲動」（同上 spy）。
  - `packages/ui/src/__tests__/archivedDetailPage.test.tsx` 新增「封存討論的結論：未轉出的刀卡帶 data-cut-todo、沒有 [data-unpromoted-cut] 提示框列」。
  - 驗證：`npm test -w packages/ui -- discussionDetailPage conclusionDoc archivedDetailPage` 中新增題目失敗、既有題目照綠。
- [ ] 3.2 讓 3.1 轉綠（design D4）：`packages/ui/src/components/discussionDocSummary.tsx` 的 `DiscussionDocData` 與 `discussionDocData` 輸入加可選 `onLocateCut`，`conclusionSummary` 在非 archived 且有 `"none"` 刀時回 `Callout tone="manual"`（標題、每刀一列 `[data-unpromoted-cut]` 含識別籤、範圍句與「前往」鈕、末行說明與指令籤）；`packages/ui/src/components/ConclusionDoc.tsx` 自 `NextCommands` 抽出並匯出 `CommandChip`、新增 `jump`／`onJumped` props 與捲動 effect（`CSS.escape` 選取 `[data-cut]`、找不到時只清請求）；`packages/ui/src/components/DiscussionDetailPage.tsx` 持有 `cutJump` 狀態（`{ id, seq }`），把 `onLocateCut` 傳進 `discussionDocData`、把 `jump`／`onJumped` 傳給 `ConclusionDoc`；`packages/ui/src/i18n.tsx` 新增 `conclusion.unpromotedCallout`、`conclusion.unpromotedHint`（tw／en 文案見 design D4）。驗證：`npm test -w packages/ui -- discussionDetailPage conclusionDoc archivedDetailPage` 全綠（既有「記錄去向 desktop-app 籤呼叫 onOpenSpec…下一步指令籤複製後剪貼簿為指令原文」不改照綠）。 <!-- speclink-task:tsk_01M4SQWTHRA95XBYVF2A4QC44B -->

## 4. 步驟列與狀態籤同色

- [ ] 4.1 先寫紅燈（design D1；spec「討論詳情頁檢視與轉出變更」的「狀態卡的保留與衍生列」「尚無結論的已轉出討論的步驟列」「已轉出且未保留的討論現站為主色」）：`packages/ui/src/__tests__/discussionDetailPage.test.tsx` 新增三題——hold 為 true 的已轉出討論，步驟列 `[aria-current="step"]` 的 className 含 `status-warning`、「已結論」站含勾選圖示；concluded 為 false 的已轉出討論，現站 className 含 `status-warning`、「已結論」站不含勾選圖示且為未到站樣式；hold 為 false、已結論的已轉出討論，現站 className 含 `primary` 不含 `status-warning`。`packages/ui/src/__tests__/archivedDetailPage.test.tsx` 既有「現站為已封存」題目不改。驗證：`npm test -w packages/ui -- discussionDetailPage` 中前兩題失敗、第三題與既有題目照綠。 <!-- speclink-task:tsk_01M4SQWTHRT1KW4QTQPC0GDMEQ -->
- [ ] 4.2 讓 4.1 轉綠（design D1）：`packages/ui/src/components/DiscussionDetailPage.tsx` 的 `DiscussionStageRow` 加可選 props `tone`（`"primary" | "warning"`，預設 primary）與 `concluded`（預設 true）——現站籤用 `tone`，`concluded` 為 false 且現站在「已結論」之後時「已結論」站畫未到站灰圈；詳情頁呼叫處傳 `tone={pill.tone === "warning" ? "warning" : "primary"}` 與 `concluded={discussion.concluded !== false}`；已封存討論頁呼叫處不改。驗證：`npm test -w packages/ui -- discussionDetailPage archivedDetailPage` 全綠。 <!-- speclink-task:tsk_01M4SQWTHRDQCZ2W22VPZJ1M75 -->

## 5. 收尾驗證

- [ ] [M] 5.1 開發環境實機驗收：以 root `npm run dev:desktop` 開桌面視窗、切到 wadpilot 專案，開 wad-plugins-merge-or-split 的討論詳情頁，確認左欄步驟列現站「已轉出」與狀態籤同為琥珀色；結論文件數字磚下有橄欖色「1 刀尚未轉出」提示框，「前往」捲到 cut 2，cut 2 為橄欖色虛線框與「尚未轉出」籤，指令籤可複製；再開一筆已封存且有未轉出刀的討論（沒有時略過此項並註記），確認刀卡有標示、沒有提示框。 <!-- speclink-task:tsk_01M4SQWTHRRC1F59XW5MB9G7S1 -->
- [ ] 5.2 全組守門：確認本變更沒有動到詞彙守門與文件連結掃描面。驗證：`npm test -w packages/ui` 全綠，且 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠。 <!-- speclink-task:tsk_01M4SQWTHRM46XAH9WG7J09R4V -->

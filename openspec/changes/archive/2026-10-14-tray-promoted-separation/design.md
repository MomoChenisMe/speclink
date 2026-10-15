## Context

- 面板唯一的呈現層是 `apps/desktop/src/panel/TrayPanel.tsx`，macOS 與 Windows 共用。中段由 `SectionCard`（檔內元件）畫四張內容卡：三張生命週期卡與「討論」卡。`SectionCard` 已負責整條標頭可點、`aria-expanded`、`tabIndex=-1`、Enter／Space 切換、箭頭（展開朝下、收合朝右）、收合時只留標頭一行，以及載入中／失敗的佔位與失敗提示。
- 已轉出現在是「討論」卡 children 的最後一段：展開時先畫 `panel-promoted-rows`（外層 `bg-muted/40`，內為 `OverflowGroup` 包 `DiscussionRow`），再畫特製收合列 `panel-promoted-toggle`（`rowClass`＋`ChevronRight`，展開時 `-rotate-90` 朝上）。討論卡的 `empty` 判斷是「討論中與已轉出皆為 0」。
- 兩份本機記憶：`speclink.tray.collapsedSections`（`sectionCollapse.ts`，四張內容卡的收合集合，預設全展開）與 `speclink.tray.promotedExpanded`（`promotedExpanded.ts`，已轉出展開與否，預設收合）。
- 分區色由 `@speclink/ui` 的 `STAGE_ICON`／`STAGE_BADGE`／`DISCUSSION_TONE` 供給；desktop-app「介面狀態語意色分層」要求「已轉出」分區與計數為中性。`tray.promotedHeader`（「已轉出」）已存在，目前只有原生選單 `tray.ts` 使用。
- 頁尾「開啟 Speclink」文字鈕用 `ArrowUpRight` 圖示。
- 來源討論：tray-promoted-separation（結論：拆成獨立卡，理由是卡片間距是面板上唯一一眼分得清的界線）。

## Goals / Non-Goals

**Goals:**

- 已轉出成為討論卡正下方的獨立卡，靠卡片間距與討論中分開。
- 已轉出卡的收合外觀與互動和其他內容卡完全一致，拿掉特製收合列。
- 預設收合與既有展開記憶不變，升級不丟使用者狀態。
- 不新增 i18n 詞條、不新增 localStorage 鍵、不新增檔案。

**Non-Goals:**

- 不動看板討論欄（`packages/ui` 的 `DiscussionColumn.tsx`、`discussion.promotedBar`、`speclink.board.promotedExpanded`）。
- 不動原生選單（`apps/desktop/src/tray.ts`）、分流判準（promoted、concluded、hold）、`TraySnapshot` 形狀、`main.tsx` 量高邏輯、溢出門檻、Rust 面板視窗。
- 不改其他四張卡的收合記憶格式；不把已轉出併進 `collapsedSections`。
- 否決的做法（討論已裁定）：留在討論卡內、收合列移到上方並改往下展開（分隔仍只靠卡內細線，且展開方向與看板相反）；只把底色加深（「還有 N 個…」像標題的主因不變）。

## Decisions

### 已轉出卡沿用 SectionCard，收合狀態接既有展開記憶

- 已轉出卡用同一個 `SectionCard` 畫，`collapsed={!promotedExpanded}`、`onToggle={togglePromoted}`；`promotedExpanded` 的 `useState(readTrayPromotedExpanded)` 與 `togglePromoted`（含 `writeTrayPromotedExpanded`）原樣保留。
- `SectionCard` 的 `id` 屬性只拿來組 testid（`panel-section-${id}`），把它的型別放寬為 `TraySectionId | "promoted"`；`TraySectionId` 與 `sectionCollapse.ts` 不動，所以 `promoted` 永遠不會寫進 `collapsedSections`。
- 替代方案：把 `promoted` 加進 `TraySectionId`、改用 `collapsedSections` 記憶——該集合預設展開，首次開面板已轉出會整組攤開，與現行預設收合相反，且舊的 `promotedExpanded` 值會被丟掉。否決。
- 替代方案：另寫一個已轉出專用卡元件——收合、鍵盤、aria、箭頭會長出第二份實作，與「與其他內容卡一致」的目標相反。否決。

### 已轉出卡只在討論清單 ready 且有料時渲染

- 渲染條件：`sectionLoad === "ready"` 且已轉出筆數大於 0；此時傳 `load="ready"`、`empty={false}`。載入中與失敗時整張不渲染（筆數未知，畫空卡或佔位都會誤導），由「討論」卡負責呈現佔位列或失敗提示。
- 位置：緊接在「討論」卡的 `SectionCard` 之後、`PanelScrollArea` 結束之前，與其他卡同屬 `panel-scroll-content` 的 `gap-2` 間距。

### 已轉出卡標頭用中性灰的討論圖示

- 圖示：`MessageSquareText`（與「討論」卡同款）加 `text-muted-foreground`；標籤：`t("tray.promotedHeader")`；計數徽章不傳 `countClassName`，沿用 Badge 的 secondary 變體（`bg-muted text-foreground`，中性灰底；專案審查規則不允許呼叫端以 className 覆蓋 @speclink/ui 元件變體的顏色）。全部是 token class，主題守門測試不受影響。
- 替代方案：`ArrowUpRight`（看板舊開關曾用）——頁尾「開啟 Speclink」已用同一圖示，一個面板裡一圖兩義。否決。
- 替代方案：給已轉出色相——違反「介面狀態語意色分層」，且與討論的桃紫互搶。否決。

### 討論卡只看討論中、已轉出列拿掉灰底

- 「討論」卡的 `empty` 改為只看討論中筆數；children 只剩討論中的 `OverflowGroup`，「還有 N 個…」自然成為最後一列。
- 已轉出卡的 children 為 `OverflowGroup` 包 `DiscussionRow`（與討論中同構），外層不再包 `bg-muted/40` 容器。
- 刪除特製收合列 `panel-promoted-toggle` 與 `panel-promoted-rows` 容器，以及其中「展開時 `-rotate-90` 朝上」的箭頭。`discussion.promotedBar` 在面板的用法隨之消失（看板仍用，詞條保留）。

## Implementation Contract

- **Behavior**：面板（macOS 與 Windows）中段順序為提案中→進行中→已就緒→討論→已轉出。已轉出卡為白底細框 12px 圓角卡，標頭為灰色討論圖示、「已轉出」、灰色計數徽章與箭頭；預設收合（只剩標頭一行、箭頭朝右）；點標頭在標頭下方展開已轉出列（箭頭朝下），再點收合；展開狀態跨面板開閉與 app 重啟保留。零筆、載入中、載入失敗時不呈現已轉出卡。「討論」卡的計數與空狀態只看討論中；收起「討論」卡不影響已轉出卡，反之亦然。原生選單與看板看不到任何差異。
- **Interface / data shape**：`TrayPanelProps`、`TraySnapshot`、`tray-panel-action` 事件種類不變。`SectionCard` 的 `id` 型別放寬為 `TraySectionId | "promoted"`（檔內元件，不匯出）。localStorage：`speclink.tray.promotedExpanded` 鍵名與 "true"／"false" 值格式不變，`speclink.tray.collapsedSections` 不變。DOM 錨點：新增（恢復）`panel-section-promoted`，其標頭觸發為該卡內的 `panel-section-toggle`；移除 `panel-promoted-toggle` 與 `panel-promoted-rows`。
- **Failure modes**：localStorage 讀寫失敗沿用 `promotedExpanded.ts` 既有行為——讀不到或壞值視為收合、寫入失敗靜默。沒有新的錯誤路徑。
- **Acceptance criteria**：
  - `apps/desktop/src/__tests__/trayPanel.test.tsx` 改寫已轉出相關題目並全綠，至少涵蓋：已轉出為獨立的 `panel-section-promoted` 卡且位於 `panel-section-discussions` 之後；預設收合、`aria-expanded=false`、已轉出列不在 DOM；點標頭展開並寫入 `speclink.tray.promotedExpanded`＝"true"、重新掛載讀回展開；箭頭展開時為 `rotate-90`（朝下）而非 `-rotate-90`；零已轉出、載入中、載入失敗皆無該卡；討論中 0＋已轉出 2 →「討論」卡為空狀態卡（計數 0）＋已轉出卡計數 2；收起討論卡時已轉出卡仍在；已轉出卡圖示與計數為中性 class、不含 `DISCUSSION_TONE` 的色相 class；已轉出列外層無 `bg-muted/40`；7 則已轉出展開為前 5＋「還有 2 個…」。
  - `npm test -w apps/desktop -- trayPanel` 全綠；`npm test -w packages/ui -- theme` 全綠（掃描面零原生色階）。
  - 手動：macOS 淺色與深色各開一次面板，對照截圖情境（討論中 11、已轉出 3）確認兩卡之間有卡片間距、已轉出卡收合與展開外觀與其他卡一致。
- **Scope boundaries**：in——`TrayPanel.tsx` 的討論卡與已轉出卡呈現、`trayPanel.test.tsx` 的對應題目、tray-status-menu 三條需求字面。out——`tray.ts` 原生選單、`promotedExpanded.ts`／`sectionCollapse.ts` 的內容、`main.tsx`、store、看板、i18n 詞典、Rust 面板視窗、Linux。

## Risks / Trade-offs

- [面板變高] 已轉出由一列變成一張卡，收合時多一個卡間距與卡框（約 14px）→ 高度上限 640 不變，超限照舊由中段捲動；量高邏輯不動。
- [面板與看板結構不再同構] 看板仍是欄底收合列、面板改為獨立卡 → 兩處的使用語意相同（預設收合、記憶、中性色），只差容器；看板分得清是因為已轉出為精簡細列、討論中為整張白卡，面板兩邊同一種列，才需要卡片間距。
- [回歸對照] `trayPanel.test.tsx` 有十多題釘住舊收合列的 testid 與箭頭方向，必須同變更改寫；沒有 golden 或 CLI 測試受影響。
- [跨平台] macOS 與 Windows 共用同一份呈現層，一次改兩平台；Linux 只有原生選單，不受影響。Windows 面板高度改變後的重新貼齊工作列邏輯已由既有需求覆蓋，不需另改。

## Context

- 面板唯一的呈現層是 `apps/desktop/src/panel/TrayPanel.tsx`：根層 `bg-background/60` 加主色→透明漸層 wash、分區卡 `bg-foreground/5 rounded-lg p-1`、列 `rounded-[5px] hover:bg-primary hover:text-primary-foreground`（站章、波次數字、複製鈕、任務數、topic 行與進度條各自掛 `group-hover:text-primary-foreground` 才在反白上看得見）、三條 `Divider`（tab 條後、討論區塊後、生命週期後）、「已轉出」為獨立的 `panel-section-promoted` 卡、專案 tab 為 24px 小方塊疊在實心主色的 tab 上。
- 入口 `apps/desktop/src/panel/main.tsx` 觀察 `panel-header`、`panel-scroll-content`、`panel-footer` 三個節點量高並 `setSize`（寬 320、上限 640）；Rust `apps/desktop/src-tauri/src/panel.rs` 套 HudWindow 材質與 13px 圓角。兩者不在本變更範圍。
- 主視窗圖示列 `apps/desktop/src/components/ProjectRail.tsx` 的專案方塊：`RAIL_SQUARE`（h-8 w-8、rounded-lg、13px 中粗字）＋作用中 `bg-primary text-primary-foreground`、其餘 `bg-muted text-muted-foreground`；首字母取 `Array.from(name.trim())[0]`；`UpdateRailButton.tsx` 與其測試自 ProjectRail 匯入 `RAIL_SQUARE`。
- 看板討論欄 `packages/ui/src/components/DiscussionColumn.tsx` 的收合列：文案 `discussion.promotedBar`（「已轉出 {n}」）＋ChevronRight（展開時 rotate-90）、展開區在列上方；展開狀態由桌面 store 持有、存 `speclink.board.promotedExpanded`。
- 守門：`packages/ui/src/__tests__/theme.test.ts` 掃三個根（含 apps/desktop/src）禁止原生色階字面，元件只能寫 token class；`apps/desktop/src/__tests__/trayPanel.test.tsx` 73 題釘住現有結構（分割線、半透明卡、主色反白、已轉出分區）。
- 面板是獨立視窗與獨立 bundle，沒有 store；localStorage 與主視窗同源可用（主題偏好已這樣讀）。`I18nProvider` 以共用元件庫的 MESSAGES 為底、APP_MESSAGES 疊上，面板可直接用 `discussion.promotedBar`。
- 討論結論指定的互動原型連結已失效；本設計以結論文字為視覺基準：面板底側欄淡灰（淺色約九成不透明）、三類白底細框 12px 圓角卡、分區標題 12px 粗體＋色相圖示＋計數、列細線分隔 hover 灰底、已轉出收進討論卡底部並記憶、專案 tab 與圖示列方塊同形。

## Goals / Non-Goals

**Goals:**

- 面板的容器語言與看板、設定頁一致：灰底上的白卡、卡內的列。
- hover 改半透明前景色灰底，列內所有符號在 hover 期間維持本色，刪掉整套反白配色。
- 專案 tab 的方塊與圖示列共用同一份樣式常數，兩處永遠同形。
- 已轉出改收合列、記憶展開狀態，與看板討論欄同構但分開存。
- 全部用 token class，theme.test 掃描面零新增原生色階。

**Non-Goals:**

- 不動原生選單（`apps/desktop/src/tray.ts`）、Rust 面板視窗、`main.tsx` 的量高邏輯、TraySnapshot 形狀。
- 不改三段式版面、溢出門檻、skeleton／載入失敗、復原卡的動作集合與回呼。
- 不改看板的收合列與記憶鍵；不加新 i18n 詞條；不放品牌標記。

## Decisions

### D1 面板底與卡片層

- 根層 `panel-root`：`flex h-screen flex-col gap-2 rounded-[13px] bg-sidebar/90 p-2 text-[13px] text-foreground`。拿掉 `bg-linear-to-b from-foreground/5 to-transparent`。13px 圓角與 Rust 的 vibrancy 半徑一致不變；`bg-sidebar/90` 淺深兩套 token 各自生效（淺色 0.985 灰、深色 0.185 灰），毛玻璃自餘下一成透出。
- 新增本地元件 `PanelCard`（取代 `SectionCard`）：`<section data-testid className={cn("flex flex-col overflow-hidden rounded-xl border border-border bg-card", className)}>`；`overflow-hidden` 讓最後一列的 hover 底色被卡角裁切。三類卡：
  - 頁首：`PanelCard testid="panel-project-card"` 包 tab 條（D3）。
  - 中段：討論卡、三張生命週期卡各一張；復原卡與 stale 列為自帶語意色邊框的卡（D5）。
  - 頁尾：`PanelCard testid="panel-actions-card"` 包四列動作。
- 刪除 `Divider` 元件與全部 `panel-divider`；`panel-header`／`panel-footer` 外層保留（量高用），只包各自的卡；中段 `panel-scroll-content` 的 `gap-2`。
- 捲動指示條不動（`right-0.5`，疊在卡右緣外的留白上）。

### D2 分區卡的標頭與列

- 標頭：共用 `SectionHeader`，`className="px-3 py-2 text-foreground"`（`cn` 以後者覆蓋元件預設的 muted 字色），其下有內容（列、骨架、失敗提示、收合列）時再加 `border-b border-border/70`；空狀態卡（計數 0、無收合列）維持 `min-h-12 justify-center`、無底線。計數徽章與色相圖示沿用 `STAGE_BADGE`／`STAGE_ICON`／`DISCUSSION_TONE`，不動。
- 列：`rowClass = "flex w-full items-center gap-2 border-b border-border/70 px-3 py-1.5 text-left last:border-b-0 hover:bg-foreground/5"`（列內已無 group-hover 配色，不再是 group）。不再有列圓角；hover 值取與 Button 的 subtle 變體同一個 `bg-foreground/5`（清單列的 3% 在 12px 字的小列上太淡）。變更列、討論列、溢出列（「還有 N 個…」／「收合」）、四列動作與已轉出收合列全部共用。
- 刪掉反白配色：列尾直接用 `CopyButton`（`tabIndex={-1}`，原 `RowCopyButton` 轉手層一併拿掉），不再傳 `inverted`；`STAMP_HOVER` 常數與所有 `group-hover:text-primary-foreground*` 類別（波次數字、站章、worktree 標記、任務數、topic 行、溢出標籤、動作列圖示）刪除；進度條軌道維持 `bg-foreground/10`、填色維持 `STAGE_BAR[stage]`，刪掉 hover 時的軌道與填色改色。被擋列改以 `[&>*]:opacity-60` 淡化列的內容——列本身降透明度會連底部分隔線一起變淡。
- 共用元件 `packages/ui/src/components/CopyButton.tsx` 的 `inverted` 屬性因此沒有消費者：連同 `copyButton.test.tsx` 的對應題目一併移除（因本變更孤兒化的死碼）。

### D3 專案 tab 與圖示列方塊同形

- 新增 `apps/desktop/src/components/projectTile.ts`，零依賴的常數模組：
  - `RAIL_SQUARE`（自 ProjectRail 搬過來，字串不變）
  - `PROJECT_TILE_ACTIVE = "bg-primary text-primary-foreground"`
  - `PROJECT_TILE_IDLE = "bg-muted text-muted-foreground"`
  - `ADD_TILE = "border border-dashed border-border bg-transparent text-muted-foreground"`
  - `projectInitial(name: string): string`＝`Array.from(name.trim())[0]?.toUpperCase() ?? ""`
- `ProjectRail.tsx` 改引用這四個常數與函式；`UpdateRailButton.tsx` 與其測試改為直接自 `projectTile` 匯入 `RAIL_SQUARE`（ProjectRail 不再轉出）；圖示列非作用中方塊在 `PROJECT_TILE_IDLE` 之後保留自己的 `hover:bg-muted/70`。
- 面板 tab（`role="tab"`，仍為 div、`tabIndex={-1}`）：tab 與「加入專案」共用容器 `TAB_ITEM = "flex shrink-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 hover:bg-foreground/5"`，tab 再加 `w-16`，並帶 `title={tab.name}`（名稱截斷時滑過可見全名）；內含方塊 `cn(RAIL_SQUARE, active ? PROJECT_TILE_ACTIVE : PROJECT_TILE_IDLE)`——方塊內依序為切換中 spinner（`data-tab-pending`，h-4 w-4 與圖示列同尺寸）、remote 狀態圖示（`RemoteTabIcon`，改 h-4 w-4）、首字母（`projectInitial`）；方塊下專案名 `max-w-14 truncate text-[11px] leading-none`，作用中 `text-foreground font-medium`、其餘 `text-muted-foreground`；remote 非 ready 再一行 `text-[9px]` 狀態字、色調沿 `tabStatusTone`。
- 選取只由方塊承載：作用中不論 ready／restoring／error 皆為實心主色方塊（與圖示列一致），tab 本體不再有 `border-primary`／`border-primary/60`、不再整塊塗主色。
- 「加入專案」：方塊 `cn(RAIL_SQUARE, ADD_TILE, "group-hover:bg-muted group-hover:text-foreground")`＋Plus h-4 w-4，文字在下且不截斷；容器 `cn(TAB_ITEM, "group min-w-16")`——英文「Add project」比 tab 寬，以最小寬度對齊 tab、隨文字伸長。
- tab 條容器 `panel-project-tabs`：`flex gap-1 overflow-x-auto p-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden`，放在 `panel-project-card` 裡；`role="tablist"` 與 aria 標籤不變。

### D4 已轉出收合列與記憶

- 新增 `apps/desktop/src/panel/promotedExpanded.ts`：
  - `export const TRAY_PROMOTED_EXPANDED_STORAGE_KEY = "speclink.tray.promotedExpanded"`
  - `readTrayPromotedExpanded(): boolean`——`localStorage.getItem(key) === "true"`，取不到或拋錯回 false。
  - `writeTrayPromotedExpanded(expanded: boolean): void`——`setItem(key, String(expanded))`，拋錯吞掉。
  - 與看板的 `speclink.board.promotedExpanded` 刻意分鍵：兩個視窗、兩個把手，互不干擾。
- `TrayPanel` 以 `useState(readTrayPromotedExpanded)` 持有，點按收合列時 `set` 並 `write`。面板每次開啟都是同一個 webview（lazy 建一次），重啟 app 後由 localStorage 讀回。
- 討論卡（`panel-section-discussions`）內容順序：標頭「討論 N」（N＝討論中筆數）→ 討論中列（`OverflowGroup`）→ 已轉出有料時：展開中先渲染 `panel-promoted-rows`（`bg-muted/40`，內為 `OverflowGroup` 包的 `DiscussionRow`，與討論中列同構）→ 收合列 `panel-promoted-toggle`：`role="button"`、`aria-expanded`、`tabIndex={-1}`、Enter／Space 同點按、`rowClass` 加 `text-xs font-medium text-muted-foreground`，左文字 `t("discussion.promotedBar").replace("{n}", …)`、右 ChevronRight（展開 rotate-90）。計數在文字裡、中性配色，不給色相徽章（desktop-app「介面狀態語意色分層」）。
- 討論中為 0、已轉出大於 0：標頭「討論 0」＋收合列，不走空狀態卡；兩者皆 0 才是空狀態卡。載入中或載入失敗不渲染收合列（同舊規則）。
- 刪除獨立的 `panel-section-promoted` 卡與 `tray.promotedHeader` 在面板的用法（該詞條仍由原生選單 `tray.ts` 使用，保留）。
- 展開改變中段內容高度，`main.tsx` 既有的 ResizeObserver 觀察 `panel-scroll-content`，自動重量高；不需改入口。

### D5 復原卡與 stale 列

- `PanelRecoveryCard`：`cn("rounded-xl border p-3", SEMANTIC_SURFACE[tone], "bg-card")`（語意色只留邊框，底色回白卡）；資訊 `dl` 的底改 `bg-muted/50`；三顆動作改共用 `Button`——重新連線／重新登入 `variant="default" size="sm" className="col-span-2"`，查看問題／伺服器設定 `variant="outline" size="sm"`，全部 `tabIndex={-1}`；圖示與文案不變。
- stale 狀態列 `panel-stale-status`：`rounded-xl border px-3 py-2`＋`SEMANTIC_SURFACE.warning`（琥珀淡底保留——與專案欄的提示卡同語意）；「重新登入」鈕改 `Button variant="outline" size="sm" tabIndex={-1}`。

### D6 測試與規格字面對齊

- `trayPanel.test.tsx` 改動清單：
  - 刪：「分割線恰三條」、「hover 反白時兩章隨列改前景色」、「hover 反白時波次數字隨列改前景色」、「作用中非 ready 分頁：選取以主色外框表達」、「無已轉出討論時不出現『已轉出』分區」。
  - 改：三段式版面的分割線斷言→三類卡（`panel-project-card`、`panel-actions-card`、各 `panel-section-*` 皆含 `bg-card`、`border-border`、`rounded-xl`）；「分區卡片化」的 `bg-foreground/5`／`rounded-lg`→同上；「根層漸層為中性」→根層無 `bg-linear-to-b` 且含 `bg-sidebar/90`；「毛玻璃補光層」→ `/bg-sidebar\/\d+/`；「作用中 tab 實心主色底」→作用中 tab 內方塊含 `bg-primary`、非作用中方塊含 `bg-muted`；「討論分流」→已轉出列在展開後的 `panel-promoted-rows` 內、不存在 `panel-section-promoted`；複製鈕題目去掉 `inverted` 註解與 `group-hover:text-primary-foreground` 斷言。
  - 新增：收合列預設收合（有已轉出時出現、`aria-expanded=false`、已轉出列不在 DOM）；點按展開並寫入 `speclink.tray.promotedExpanded`＝"true"；重新掛載讀回展開；零已轉出無收合列；討論中 0＋已轉出 2 → 標頭計數 0＋收合列；展開區 7 則→前 5＋「還有 2 個…」；記憶鍵壞值→收合；tab 方塊 class 含 `RAIL_SQUARE` 全部 token；加入專案方塊含 `border-dashed`；列 hover class 為 `hover:bg-foreground/5` 且無 `hover:bg-primary`；站章與波次數字無 `group-hover:text-primary-foreground`。
- 規格字面：tray-status-menu「面板交接主視窗後滑鼠互動完整」的兩個 scenario 把「抽屜」改為「詳情頁」（關閉方式改為 Esc 或麵包屑），純字面對齊 cut 3b，無程式碼對應。
- 守門：`npm test -w packages/ui -- theme copyButton`（掃描面零原生色階、`inverted` 題目移除後全綠）、`npm test -w apps/desktop -- trayPanel projectRail updateRailButton`；詞彙守門不受影響（無 i18n 改動）。

## Implementation Contract

- **Behavior**：macOS 使用者點系統匣圖示後看到——淡灰補光底上的白卡：頂端 tab 條卡（方塊與主視窗圖示列同形，作用中實心主色）、中段討論卡（底部「已轉出 N ›」收合列，預設收合、記憶展開）與三張生命週期卡、底部四列動作卡；列 hover 灰底、站章與波次本色；復原卡與 stale 列為白卡或琥珀淡底卡。原生選單使用者看不到任何差異。
- **Interface / data shape**：`TrayPanelProps`、`TraySnapshot`、`tray-panel-action` 事件種類不變。新匯出：`apps/desktop/src/components/projectTile.ts` 的 `RAIL_SQUARE`、`PROJECT_TILE_ACTIVE`、`PROJECT_TILE_IDLE`、`ADD_TILE`、`projectInitial`；`apps/desktop/src/panel/promotedExpanded.ts` 的 `TRAY_PROMOTED_EXPANDED_STORAGE_KEY`、`readTrayPromotedExpanded`、`writeTrayPromotedExpanded`。移除：`CopyButton` 的 `inverted` 屬性。DOM 錨點新增 `panel-project-card`、`panel-actions-card`、`panel-promoted-toggle`、`panel-promoted-rows`；移除 `panel-divider`、`panel-section-promoted`；其餘 testid 不變（含量高用的三個）。
- **Failure modes**：localStorage 讀寫失敗靜默、視為收合；`discussion.promotedBar` 由共用詞典供給，缺鍵時顯示鍵名（與既有 i18n 行為一致）。沒有新的錯誤路徑。
- **Acceptance criteria**：D6 列出的測試全綠；`theme.test.ts` 掃描面無新增原生色階；`speclink validate tray-panel-restyle` 通過；手動在 macOS 淺色與深色各開一次面板，對照 spec「面板樣式（macOS）」的「分區卡與列的外觀」「專案 tab 方塊與圖示列同形」「深色背景下面板亮度不明顯下沉」與「討論列表」的「面板的已轉出收合列預設收合並記憶展開」。
- **Scope boundaries**：in——`TrayPanel.tsx`、`projectTile.ts`、`promotedExpanded.ts`、`ProjectRail.tsx` 與 `UpdateRailButton.tsx`（含其測試）的匯入搬遷、`CopyButton.tsx` 的屬性移除、對應測試、tray-status-menu 五條需求字面。out——`tray.ts`、`panel.rs`、`main.tsx`、store、看板、i18n 詞條、Windows／Linux。

## Risks / Trade-offs

- 補光九成讓毛玻璃只剩一成透感——討論裁定值；真機若太悶可在 85–90 之間調，spec 寫「約九成」容許微調。
- `hover:bg-foreground/5` 比清單列的 3% 深一級——面板列小、字 12px，取與 subtle 按鈕同值；兩處不統一是刻意的。
- 卡片邊框與內距讓面板自然高度比現在多約 40–60px——上限 640 不變，超限仍由中段捲動；量高邏輯不動。
- `CopyButton` 移除屬性是跨套件的小改——只有面板用過，移除後若未來再有反白列需求再加回；留著是死碼。

## Migration Plan

無資料遷移；`speclink.tray.promotedExpanded` 鍵首次缺席即收合。測試與程式碼同一變更內對齊。

## Open Questions

無。

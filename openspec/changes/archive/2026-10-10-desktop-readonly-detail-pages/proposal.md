## Summary

把桌面 app（apps/desktop）的「規格」與「已封存」兩種唯讀右側面板改成佔滿主區的詳情頁：規格詳情頁把正式規格逐條需求分段呈現並附需求目錄與溯源分頁；已封存詳情頁與變更／討論詳情頁同骨架、唯讀。兩者沿用 `desktop-detail-page` 落地的頁層、麵包屑、‹ n／m › 與「開啟 ▾」，宿主頁（規格頁、已封存頁、手冊頁）維持不變。

## Motivation

cut 3b 的前半把看板宿主的變更與討論詳情改成頁，主區剩下規格與已封存兩種面板仍以 Sheet 滑出、蓋在清單上。使用者在互動原型確認了兩張畫板：規格詳情頁的需求以「Requirement 標籤＋標題＋來源錨點、需求文、左線縮排的 Scenario」分段、寬於 1200px 時左側列需求目錄、分頁為規格／溯源；已封存詳情頁與變更詳情頁同骨架且唯讀。做完這一半，桌面主區就沒有任何面板，Sheet 原語在桌面退場。

目標使用者是讀規格、追某條需求來自哪個變更、回看已封存變更與討論的開發者與 PO／PM；對應 trace（溯源）與 archive 之後的回看情境。

## Proposed Solution

- **規格詳情頁**：麵包屑「專案名 / 規格 / capability」＋‹ n／m ›（規格頁目前過濾後的順序；自手冊頁開啟時為全部規格的順序）；標題列（22px 等寬名稱＋複製）、灰字 meta 列（N 條需求 · 來自 M 個已封存變更 · 最近更新 日期（變更名））、右端「搜尋需求」輸入（依需求標題過濾分段與目錄）；內容卡標頭分頁「規格 N」「溯源 M」＋「開啟 ▾」（`openspec/specs/<capability>/spec.md`）；「規格」分頁為 Purpose 段，其後每條需求一段：「Requirement」小標＋標題＋右端「來自 <變更> · 日期」錨點（取該需求之後的 @trace 區塊）、需求文（SHALL／MUST 等關鍵字以等寬灰底標示）、Scenario 以左側細線縮排區塊呈現；頁寬於 1200px 時左側 200px 需求目錄列出全部需求標題、點擊捲至該段、捲動時高亮目前段；「溯源」分頁列出 @trace 來源變更：封存清單命中者為可點列（副標封存日期、依封存日期升冪、點擊開該封存變更的詳情頁），未命中者不可點、副標「無封存記錄」、排在最後。無 `### Requirement:` 標題的規格整篇以單一 markdown 檢視退回。
- **已封存詳情頁**：封存變更——標題列（變更名＋複製含日期前綴的封存目錄名＋審查／驗證結局籤）、出身列（建立者、建立日、封存日、「來自」來源討論籤）、無動作區；「狀態」卡三列（任務 n/m、品質關卡結局、封存日期）；分頁 提案／設計／任務／規格（＋條件式 審查／驗證）＋「開啟 ▾」（`openspec/changes/archive/<dated>/…`）；任務分頁唯讀、無把手與工具列。封存討論——slug 標題＋複製＋「已封存」籤＋改進籤、topic、出身列（建立者、建立日、封存日）；「狀態」卡：階段列現站「已封存」、衍生變更列三態籤；分頁 結論／討論過程／背景／衍生變更＋「開啟 ▾」只有「複製內容」。
- **宿主與導覽**：規格頁與已封存頁的清單元件回報目前過濾後順序供 ‹ ›；自手冊頁出處開啟規格詳情頁時宿主維持手冊頁、Esc 回手冊；自規格詳情頁溯源列或已封存討論的衍生籤開啟已封存詳情頁時宿主不切頁。
- **返回**：詳情頁開啟時麵包屑左側出現返回鈕（「返回<頁名>」），⌘[、⌘←（非 macOS 為 Alt+←）、滑鼠側鍵「上一頁」與觸控板兩指向右滑動（左緣出現「←」指示，滑過門檻放開即返回）等效，皆回宿主清單頁（與 Esc 同一動作）；經專案欄切換主區頁面時四種詳情頁一律先關閉。
- **退場**：`SpecDrawer`、`ArchivedDrawer` 刪除；桌面不再引用 Sheet（原語留在 packages/ui 供 server-web）。

## Non-Goals

- 變更與討論詳情頁、頁層、快捷鍵、「開啟 ▾」選單本身——前半 change。
- 規格頁、已封存頁、手冊頁的清單版面與換頁——cut 3a 已定。
- 規格內文的 markdown 渲染規則（alert、表格、程式碼）——沿用共用 Markdown 元件。
- 已封存討論記錄的「在 Finder 顯示」：封存檔名帶封存日期而清單資料不帶，只提供複製內容。
- 從 packages/ui 刪除 Sheet 原語。

## Alternatives Considered

- 規格全文整篇 markdown 渲染、只在標題上掛錨點——拿不到「來自 <變更> · 日期」錨點與 Scenario 縮排區塊，使用者確認的原型需要分段。
- 需求目錄用 JS 量測視窗寬度——以 CSS 容器查詢（`@container` 與 `@min-[1200px]`）即可，不需監聽。
- 已封存變更與討論各做一個頁元件——兩者共用標題列、出身列、狀態卡與分頁骨架，一個元件兩個變體較少重複。

## Impact

- Affected specs: desktop-app（規格與已封存相關 8 條需求改為詳情頁語意，其中名稱含「抽屜」者以 REMOVED＋ADDED 更名；「詳情頁的導覽與返回」MODIFIED 加返回鈕、返回快捷鍵、滑鼠側鍵與規格／已封存的序列規則）、desktop-manual-page（「內頁渲染與出處跳規格」的出處開規格詳情頁）
- Affected code:
  - New: `packages/ui/src/specSections.ts`、`packages/ui/src/lib/useGenerationLoad.ts`、`packages/ui/src/components/SpecDetailPage.tsx`、`packages/ui/src/components/ArchivedDetailPage.tsx`、`packages/ui/src/__tests__/specSections.test.ts`、`packages/ui/src/__tests__/specDetailPage.test.tsx`、`packages/ui/src/__tests__/archivedDetailPage.test.tsx`
  - Modified: `packages/ui/src/index.ts`、`packages/ui/src/i18n.tsx`、`packages/ui/src/components/SpecList.tsx`、`packages/ui/src/components/ArchivedList.tsx`、`packages/ui/src/components/ManualToc.tsx`、`packages/ui/src/components/Markdown.tsx`、`packages/ui/src/components/DetailHeader.tsx`、`packages/ui/src/components/DiscussionDetailPage.tsx`、`packages/ui/src/components/ListRow.tsx`、`packages/ui/src/__tests__/listRow.test.tsx`、`packages/ui/src/__tests__/specList.test.tsx`、`packages/ui/src/__tests__/archivedList.test.tsx`、`apps/desktop/src/App.tsx`、`apps/desktop/src/index.css`、`apps/desktop/src/components/DocumentActionsMenu.tsx`、`apps/desktop/src/components/DetailNav.tsx`、`apps/desktop/src/components/SwipeBack.tsx`（新）、`apps/desktop/src/__tests__/swipeBack.test.tsx`（新）、`apps/desktop/src/components/TitleBar.tsx`、`apps/desktop/src/i18n/messages.ts`、`apps/desktop/src/__tests__/detailNav.test.tsx`、`apps/desktop/src/adapter/fsActions.ts`、`apps/desktop/src/__tests__/App.test.tsx`、`apps/desktop/src/__tests__/documentActionsMenu.test.tsx`、`apps/desktop/src/__tests__/fsActions.test.ts`、`apps/desktop/core/src/query.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src/store.ts`、`apps/desktop/src/__tests__/store.test.ts`、`packages/ui/src/__tests__/verifyBadge.test.tsx`、`packages/ui/src/__tests__/improveBadge.test.tsx`、`packages/ui/src/__tests__/reviewBadge.test.tsx`
  - Removed: `packages/ui/src/components/SpecDrawer.tsx`、`packages/ui/src/components/ArchivedDrawer.tsx`、`packages/ui/src/__tests__/specDrawer.test.tsx`、`packages/ui/src/__tests__/archivedDrawer.test.tsx`、`packages/ui/src/lib/useLingering.ts`

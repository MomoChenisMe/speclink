## Context

討論詳情頁（`packages/ui/src/components/DiscussionDetailPage.tsx`）與已封存討論頁（`packages/ui/src/components/ArchivedDetailPage.tsx`）共用三個部件：左欄的階段步驟列 `DiscussionStageRow`、結論渲染器 `ConclusionDoc`（`packages/ui/src/components/ConclusionDoc.tsx`）、以及閱讀卡標題區的推導入口 `discussionDocSummary`（`packages/ui/src/components/discussionDocSummary.tsx`）。

- 身分區狀態籤由 `statusPill()` 推出：promoted 且尚無結論、或 promoted 且保留在途時為 warning，其餘 promoted 與 concluded 為 primary，open 為 neutral。
- `DiscussionStageRow({ current })` 只收現站序號：現站固定 `tone="primary"`，現站之前的站一律打勾。詳情頁以 `stationIndex`（promoted 為 2）呼叫；已封存討論頁以 `current={3}` 呼叫。
- `ConclusionDoc` 的 `CutCardView` 以 `stageOf(name)` 查變更的階段籤：查不到（不在 `promoted_to`）就不畫任何東西。
- `conclusionSummary` 只出數字磚，正典寫「結論文件 SHALL NOT 有提示框」。任務文件的手動任務提示框（`changeDocSummary.tsx`）是 `Callout tone="manual"`，每列有「前往 →」，經 `useDocNav` 的 `locateTask` 請求捲到任務列。
- 本變更全在 `packages/ui`（呈現層），不碰任何 crate、Host 或 Protocol 契約；local 與 remote 的討論資料走同一條 `DiscussionDocData`，沒有兩套實作。

## Goals / Non-Goals

**Goals:**

- 步驟列現站與身分區狀態籤同色；尚無結論的已轉出討論，步驟列不宣稱「已結論」。
- 結論文件標出還沒轉出的刀：刀卡橄欖色虛線框與「尚未轉出」籤，live 討論另有「N 刀尚未轉出」提示框與轉出指令。
- 未轉出的判定只有一個來源，提示框與刀卡不會互相矛盾。

**Non-Goals:**

- 衍生變更文件（`PromotedDoc`）不列預定中、還沒轉出的刀。
- 不推算還欠幾刀，不依子變更狀態推論系列進度。
- 身分區狀態籤判準、看板討論卡、系統匣不改；引擎與記錄格式不改。

## Decisions

### D1 步驟列現站色調與已結論站

`DiscussionStageRow` 新增兩個可選 props：`tone`（`"primary" | "warning"`，預設 `"primary"`）與 `concluded`（布林，預設 `true`）。

- 現站籤用 `tone`；第 1 站（已結論）在 `current > 1` 且 `concluded === false` 時畫成未到站的灰圈，其餘站的規則不變。
- 詳情頁傳 `tone={pill.tone === "warning" ? "warning" : "primary"}` 與 `concluded={discussion.concluded !== false}`。只映射 warning：`statusPill()` 對 open 討論回 neutral，若直接傳 `pill.tone` 會讓「討論中」現站從主色變灰，那是沒被要求的改變。
- 已封存討論頁維持 `current={3}`、不傳新 props，行為不變。
- 替代方案：把 `statusPill()` 的結果整個傳進去——會把 open 的 neutral 一起帶進步驟列，否決。

### D2 未轉出判定的單一來源

`packages/ui/src/discussionModel.ts` 新增純函式 `cutPromotion(cut: CutCard, isPromoted: (name: string) => boolean): "promoted" | "partial" | "none" | "unknown"`。

- 只看粗體內的變更名（討論技能的寫法 ``**cut N `change-name`**``）。`CutCard` 新增 `namesInBold`：粗體內有反引號片段時為 true。粗體內沒有名稱時 → `"unknown"`（不標示），包含刀列沒寫變更名、以及舊式記錄把名稱寫在粗體之後的情形——後者的反引號片段常是識別字或檔名（例：已封存的 improve-lifecycle-layer 的「**刀一**：`KeyLines` 原語＋`DiscussionHead`」），拿來判定會把已轉出的刀誤標成未轉出。名稱籤與階段籤的呈現不受影響。
- 全部名稱 `isPromoted` → `"promoted"`；全部不是 → `"none"`；其餘 → `"partial"`。
- `isPromoted` 只寫一次：`ConclusionDoc.tsx` 匯出 `cutPromotionOf(cut, stageOf)`，以「變更名查得到階段籤」為已轉出。刀卡傳結論文件收到的 `stageOf`，提示框傳 `rowStage(data.rows)`——兩者都是衍生列（`PromotedRow[]`，即 `promoted_to` 推出的列，含已封存與已刪除）推出的同一個查表。
- 刀卡與提示框都經這支函式，提示框列出的刀＝畫成虛線框的刀（都是 `"none"`）。
- 替代方案：刀卡用 `stageOf` 為 null、提示框自己比對 `rows`——兩處各寫一次規則，日後改一邊會分岔，否決。

### D3 刀卡的未轉出標示

`CutCardView` 依 `cutPromotionOf` 的結果畫：

- `"none"`：卡的外框改虛線並套 `SEMANTIC_SURFACE.manual`（`border-status-manual/40 bg-status-manual/10`，再加 `border-dashed`），卡頭的刀編號籤之後加 `StatusPill tone="manual"`「尚未轉出」；卡帶 `data-cut-todo` 屬性。變更名籤維持不可點、無階段籤。
- `"partial"`：外框照舊；每個未轉出的變更名籤後接同一顆「尚未轉出」籤（取代原本的空白）。
- `"promoted"` 與 `"unknown"`：照現行畫法。
- `StatusPill` 的色調表（`packages/ui/src/components/ui/status-pill.tsx`）新增 `manual: "bg-status-manual/12 text-status-manual"`；呼叫端不以 className 覆蓋色調。
- 已封存討論頁同一個 `ConclusionDoc`，刀卡標示照常出現。

### D4 結論提示框與「前往」

- `DiscussionDocData` 新增可選欄位 `onLocateCut?: (id: string) => void`；只有討論詳情頁傳入，已封存討論頁不傳。
- `conclusionSummary` 在 `data.status !== "archived"` 且至少一刀為 `"none"` 時回傳 `Callout tone="manual"`。只排除已封存：引擎只拒絕從已封存的記錄轉出，已轉出、未保留的 live 討論照樣能轉出下一刀，提示框照列。
  - 標題 `conclusion.unpromotedCallout`（tw「{n} 刀尚未轉出」、en「Cuts not promoted yet: {n}」——英文把數字放在冒號後，n＝1 時不會出現「1 cuts」）。
  - 每刀一列 `[data-unpromoted-cut]`：刀編號籤、範圍句（`InlineMarks`）、ghost 鈕「前往 →」（沿用 `tasks.locate` 文案），點擊呼叫 `onLocateCut(cut.id)`；`onLocateCut` 缺席時無鈕。
  - 末行 `conclusion.unpromotedHint`（tw「這幾刀還沒有變更。用下面的指令轉出下一刀：」、en「These cuts have no change yet. Promote the next one with:」）接一顆指令籤 `/speclink-propose --from-discussion <slug>`，slug 取 `data.context.names.self`；`self` 缺席時整行不出現（刀的列照列），避免指令裡出現 `undefined`。
- 指令籤與「下一步」欄位的指令籤是同一個樣子：自 `NextCommands` 抽出 `CommandChip({ command })`（等寬籤＋`CopyButton`），放進共用零件檔 `DocParts.tsx` 匯出（與 `NameChip`、`NAME_CHIP_CLS` 同處），兩處共用。
- 「前往」：詳情頁持有 `cutJump` 狀態（`{ id, seq }`，seq 遞增，同一刀可重複跳），`onLocateCut` 設定它；`ConclusionDoc` 新增 `jump` 與 `onJumped` props，在 effect 中把 `[data-cut="<id>"]`（`CSS.escape`）捲到畫面中央後呼叫 `onJumped` 清除請求。提示框只出現在結論文件，不需要切換文件。
- 已封存討論頁：`status` 為 `"archived"`，提示框不出現。
- 替代方案：把 `cutJump` 放進 `useDocNav`——那是變更頁的跳轉狀態，討論頁沒有使用它，為一個請求引入整個 hook 不划算，否決。

## Implementation Contract

**行為**

- 討論詳情頁：身分區狀態籤為「已轉出・保留中」或「已轉出・尚無結論」時，步驟列現站「已轉出」為 warning 色；其餘狀態現站為主色。尚無結論時「已結論」站為灰圈（無勾）。
- 結論文件：整刀未轉出的刀卡為橄欖色虛線框與淡橄欖底，卡頭有「尚未轉出」籤；部分轉出的刀只在未轉出的變更名旁有該籤。
- live 討論的結論文件：至少一刀整刀未轉出時，數字磚之下有橄欖色提示框「N 刀尚未轉出」，列出這些刀，「前往 →」把該刀卡捲到畫面中央；末行有可複製的 `/speclink-propose --from-discussion <slug>`。全部刀都已轉出、沒有分刀、或刀列粗體內都沒寫變更名時沒有提示框。
- 已封存討論頁：刀卡標示照常；沒有提示框；步驟列不變。

**介面**

- `DiscussionStageRow({ current, tone?, concluded? })`
- `cutPromotion(cut, isPromoted) → "promoted" | "partial" | "none" | "unknown"`（`packages/ui/src/discussionModel.ts` 匯出）；`CutCard.namesInBold: boolean`
- `cutPromotionOf(cut, stageOf)`（`packages/ui/src/components/ConclusionDoc.tsx` 匯出；刀卡與提示框共用的判定入口）
- `DiscussionDocData.onLocateCut?: (id: string) => void`
- `ConclusionDoc` 新增 props `jump?: { id: string; seq: number } | null`、`onJumped?: () => void`
- `CommandChip({ command })` 自 `DocParts.tsx` 匯出
- `StatusPillTone` 新增 `"manual"`
- i18n 新增 `conclusion.unpromotedCallout`、`conclusion.unpromotedHint`、`conclusion.notPromoted`（tw「尚未轉出」、en「Not promoted」）

**失敗模式**

- 記錄切分失敗或結論缺 Decision：沒有分刀，沒有提示框與標示（沿用現行退回）。
- `data.context` 為 null：不會出結論文件，提示框不會出現。`names.self` 缺席：提示框照列刀，末行說明與指令籤不出現。
- 舊式刀列（變更名寫在粗體之後）：無從判定，不標示、不列入提示框。
- 「前往」的目標卡不在畫面（不可能：提示框只列結論文件裡的刀）：`querySelector` 回 null 時不捲動、照樣清除請求。

**驗收**

- `npm test -w packages/ui -- discussionModel statusPill conclusionDoc discussionDetailPage archivedDetailPage` 全綠，含各 task 點名的新題目。
- `[M]` 開發環境實機看 wadpilot 的 wad-plugins-merge-or-split 與一份尚無結論的已轉出討論。

**範圍**

- 範圍內：上列 `packages/ui` 的元件（含共用零件檔 `DocParts.tsx`）、純函式、i18n 與測試。
- 範圍外：`PromotedDoc`、看板討論卡、系統匣、引擎、CLI、server。

## Risks / Trade-offs

- [回歸對照] 本變更不動任何 CLI 輸出與 golden；風險只在 `packages/ui` 既有測試。→ `conclusionDoc.test.tsx` 既有「第三張變更籤不可點且無階段籤」與 `discussionDetailPage.test.tsx` 既有數字磚題目「無提示框」要隨規格改寫，task 明列，其餘題目不改全綠。
- [跨平台] 純前端呈現，Windows／macOS／Linux 行為一致；`CSS.escape` 在 Chromium 系 WebView 與 WKWebView 都可用，jsdom 也有。jsdom 沒有 `scrollIntoView`，測試要 spy／stub。
- [誤判已轉出] 刀列的變更名打錯字時，該刀會被標成「尚未轉出」。→ 這正是讀者該看到的訊號（結論寫的名字和實際轉出的不一致），不另做模糊比對。
- [舊式刀列不判定] 只認粗體內的變更名，所以名稱寫在粗體之後括號裡的刀（例：「**cut 2 外殼**（…`desktop-shell-redesign`…）」）不再標轉出狀態，即使那些名稱真的是變更名；名稱籤與階段籤照常顯示。→ 換來的是不會把識別字、檔名當成未轉出的變更；討論技能對新結論已規定粗體寫法。
- [提示框數量] 提示框從七處變八處。→ 判準沿用「只放推得出的事實」，本處由結論與 `promoted_to` 推出，與「只差最後一刀」同類。
- [storage 解耦] 不涉及規格儲存或流程抽象；資料仍來自既有 `DiscussionDocData`，local 與 remote 同一條路徑。

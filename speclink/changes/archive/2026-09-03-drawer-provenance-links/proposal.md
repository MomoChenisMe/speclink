## Why

桌面 app 的「規格 → 變更 → 討論」脈絡鏈只差一跳就閉環：規格抽屜已在內文底部列出溯源變更名，但那是一行不可點的灰字；已封存討論抽屜的卡片有衍生變更數徽章，抽屜裡卻沒有衍生變更名單。透過 AI 代理跑 SDD 的開發者、PO 與 PM 在 desktop 讀正典規格或封存記錄時，想知道「這條規格是哪個變更寫進來的、那個變更從哪份討論談出來的」，目前只能離開 GUI 改跑 `speclink trace`。引擎已有整條鏈（trace 動詞）、三跳中兩跳的 UI 已存在（手冊出處可開規格抽屜、封存變更抽屜有「來自」討論籤），補上最後一跳並統一三個唯讀抽屜的標頭文法，即可讓鏈的每一跳都可點。手冊本身維持只讀正式規格，脈絡放在一次點擊之外的既有抽屜——手冊讀者（操作者）與規格讀者（開發者／PO）各取所需。

本變更源自討論 `spec-drawer-trace-links`；步驟 3 的規格掃描命中 `desktop-app`（規格抽屜的溯源資訊、已封存項目以抽屜檢視、detail 抽屜互斥、變更與討論抽屜開啟時底層落回看板）、`desktop-manual-page`（出處可點開規格抽屜）、`discussion-docs`（討論與變更鏈結雙向可查）與 `trace-verb`（演進鏈）；只有 `desktop-app` 的需求字面需要改，其餘為既有承諾不動。

## What Changes

- **規格抽屜標頭改為兩層**：標題列（capability 名＋複製名稱鈕，與規格卡的複製鈕同款）與出身列（「來自」＋第一顆溯源變更籤＋「+N」溢出浮層）。籤重用變更詳情抽屜與已封存抽屜共用的來源連結籤元件；第一顆籤為最早封存的變更（此 capability 的出身），其餘依封存日期收進浮層，浮層項副標為封存日期。點籤開啟該封存變更的唯讀抽屜（依 detail 抽屜互斥規則規格抽屜關閉、底層頁面維持不變）。無對應封存記錄的變更名呈現為不可點籤，副標「無封存記錄」。
- **規格抽屜內文底部的「來源變更：」灰字行移除**，對應的 i18n 詞條一併清除（人眼 GUI 變化，無 CLI 或 `--json` 影響）。
- **已封存討論抽屜出身列新增「衍生」列**：「衍生」＋第一顆衍生變更籤＋「+N」浮層，資料自封存討論清單項既有的 promotedTo 派生，抽屜零新查詢。點籤三態：子變更已封存→開其封存抽屜；仍活躍→開其詳情抽屜（底層依正典落回看板）；兩處皆無→不可點籤、副標「無封存記錄」。無衍生變更時整列缺席。
- **來源連結籤元件補「不可點」狀態**：籤面灰化、無 hover、無點擊，提示仍顯示名稱與副標；規格抽屜與已封存討論抽屜共用同一機制。
- **詞彙表**：`openspec/LANGUAGE.md` 的「衍生變更」詞條補用法註記——抽屜出身列的標籤縮寫為「衍生」，與「來自」「同源」同為兩字關係詞；不新增詞條。

影響範圍限於前端：`packages/ui`（三個元件、i18n、測試）與 `apps/desktop`（App 接線與測試）。無 Rust crate 改動、無 CLI 指令或 `--json` 輸出變更、無設定欄位變更、不影響任何生成的技能或 Agent 指令。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`：「桌面 app 呈現 change 與 spec 的清單與內容」——規格抽屜的溯源資訊自內文底部的文字行改為標頭標題列（含複製名稱鈕）與出身列的可點變更籤、跳轉與不可點狀態；「已封存項目以抽屜檢視」——封存討論抽屜出身列新增「衍生」變更籤列與三態跳轉。

## Impact

- Affected specs: `desktop-app`（兩條 MODIFIED 需求，scenario 名稱保留）
- Affected code:
  - Modified:
    - packages/ui/src/components/SpecDrawer.tsx（標頭兩層、移除 footer、以 host 傳入的封存清單解析籤）
    - packages/ui/src/components/ArchivedDrawer.tsx（討論型別的出身列加「衍生」列）
    - packages/ui/src/components/SourceDiscussionChip.tsx（來源連結項補不可點狀態）
    - packages/ui/src/i18n.tsx（新增「衍生」「無封存記錄」詞條；移除「來源變更：」與分隔符詞條）
    - apps/desktop/src/App.tsx（規格抽屜傳入封存清單與開啟封存變更的回呼；已封存討論抽屜傳入衍生變更三態清單）
    - packages/ui/src/__tests__/specDrawer.test.tsx（footer 斷言改為標頭籤斷言）
    - packages/ui/src/__tests__/archivedDrawer.test.tsx（衍生列三態）
    - apps/desktop/src/__tests__/App.test.tsx（接線）
    - openspec/LANGUAGE.md（「衍生變更」詞條用法註記）
  - New: （無）
  - Removed: （無）
- 相容性影響：GUI 規格抽屜的內文底部不再有「來源變更：」文字行，改由標頭籤承載；既有測試對該字面的斷言同批更新。CLI 人眼輸出與 `--json` 皆不變，golden 不受影響。

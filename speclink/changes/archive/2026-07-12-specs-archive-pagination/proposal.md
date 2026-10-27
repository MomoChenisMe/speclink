## Why

規格頁與已封存頁的清單一律最舊在前（封存變更按封存目錄名升冪、封存討論按檔名升冪、規格按名稱字母序），且已封存頁「變更／討論」兩節上下堆疊——封存量已達 40 變更＋26 討論且持續成長，找一筆最近封存的討論要捲過整節變更再從最舊往下捲。另外抽屜全螢幕時內文維持行寬上限卻靠左貼齊，右側大片死區看起來像破版（行寬上限本身是 drawer-document-readability 定案的正確可讀性決策，錯的只有靠左）。本變更出自已結論討論 specs-archive-pagination，目標使用者為透過桌面 app 檢視 SDD 工作狀態的開發者／PO／PM，情境是規格頁、已封存頁與抽屜閱讀的日常查找與回顧，對應 workflow 全階段的檢視面。

## What Changes

- 三清單改最新在前，排序在前端呈現層：封存變更＝封存日期新→舊、封存討論＝建立日期新→舊、規格＝最後修改時間新→舊（mtime 缺席排最後、名稱字母序決勝）。
- 規格頁與已封存頁清單加純前端換頁：每頁 20 筆、僅一頁時隱藏換頁控制列、搜尋字串變更即跳回第 1 頁；UI 文案僅用「上一頁／下一頁／第 N 頁」。
- 已封存頁「變更／討論」雙節堆疊改為頁內子頁籤＋筆數徽章：搜尋框共用、同時過濾兩節，徽章各自顯示命中數（在任一子頁籤可見另一節有幾筆命中），兩子頁籤各自獨立換頁。
- 抽屜閱讀欄置中：容器寬於行寬上限時（含全螢幕）留白均分兩側——明文推翻 drawer-document-readability 的「內容靠左」半句；spec-archive-drawer 落地後新增的規格抽屜與封存抽屜屬同一共用閱讀面，實作時一併套用。
- 行寬上限值由 72ch 放寬為 96ch（約 48 全形字/行）：置中落地後使用者檢視實機留白仍嫌多，2026-07-12 裁定放寬（推翻本提案原 Non-Goals「行寬上限值本身不動」半句）；「行長逾 100 全形字不可讀」的硬上限維持不動，96ch 仍在其內。
- openspec/LANGUAGE.md 收錄詞彙裁定：「分頁」保留給 tabs 語意；pagination 於 artifacts 散文稱「換頁」，UI 文案不另造名詞。

時序前提：本變更與 spec-archive-drawer 動到相同清單元件，排在其落地之後動工；開工前先執行 drift 檢查 delta 假設。

相容性影響：speclink-core／speclink-cli 的人眼與 --json 輸出一律不動，回歸對照不受影響；桌面清單 payload 欄位不增不減（排序與換頁純屬前端呈現），桌面 core（apps/desktop/core）亦不動。

## Non-Goals

- 後端換頁 IPC 或清單 payload 欄位擴充——量級（數十筆）不需要、清單資料已全量在記憶體（討論中已否決）。
- 引擎層排序改動——CLI 輸出（含 discuss list 與 list --specs 的順序）是回歸保護對象（討論中已否決）。
- 已封存頁合併單一時間軸或左右雙欄版面——變更卡與討論卡解剖不同、窄視窗爆版（討論中已否決）。
- 取消行寬上限或逾 100 全形字的行長——不可讀（討論中已否決）。（2026-07-12 修訂：原句「行寬上限值本身不動」由使用者裁定推翻，上限值放寬為 96ch，見變更內容；取消上限仍屬 Non-Goal。）
- 看板（變更欄／討論欄）的排序與換頁——看板已有拖曳排序（board_rank）語意，不屬本題。
- spec-archive-drawer 在途任務的調整；TaskList 拖曳等既有互動行為改動。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 規格頁與已封存頁清單的排序與換頁行為；已封存頁雙節版面改子頁籤；共用 markdown 閱讀容器由靠左改置中。

## Impact

- Affected specs: `desktop-app`
- Affected crate: 無（speclink-core、speclink-cli、apps/desktop/core 皆不動；純 packages/ui 前端）
- Affected code:
  - New:
    - packages/ui/src/components/ListPager.tsx
    - packages/ui/src/__tests__/listPager.test.tsx
  - Modified:
    - packages/ui/src/components/SpecList.tsx
    - packages/ui/src/components/ArchivedList.tsx
    - packages/ui/src/components/RichDetailDrawer.tsx
    - packages/ui/src/components/DiscussionDrawer.tsx
    - packages/ui/src/components/Markdown.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/specList.test.tsx
    - packages/ui/src/__tests__/archivedList.test.tsx
    - packages/ui/src/__tests__/richDrawer.test.tsx
    - packages/ui/src/__tests__/discussionDrawer.test.tsx
    - openspec/LANGUAGE.md
  - Removed: (none)

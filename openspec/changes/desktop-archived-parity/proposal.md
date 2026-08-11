## Why

desktop 的「已封存」頁停留在舊設計，與看板既有的呈現脫節：自系統匣（或任何跨頁入口）開啟變更／討論時底層頁面留在原頁不回看板、已封存的變更卡與討論卡呈現不對稱（變更只有英文名無描述、討論只有中文 topic 無 slug 錨點）、已封存抽屜的標頭缺少活躍變更抽屜已有的複製鈕與出身資訊。三者都不是新設計，而是封存側沒跟上看板既有的 anatomy（源自已結論的討論 archived-parity-and-spec-purpose，議題 1／2／3）。

## What Changes

- **抽屜開啟落回看板**：`openDetail`／`openDiscussion` 於設定抽屜狀態時一併把底層頁面切回看板（`boardView: "board"`）——涵蓋全部入口（系統匣、同源變更互跳、討論抽屜跳衍生變更、封存前「去蓋章」），不只系統匣一條路徑。系統匣只列活躍項，目的地無「該落在已封存頁」的分支。
- **已封存卡片改雙行 anatomy**：封存討論卡以 slug 為標題（等寬強調、附複製鈕）、topic 降為描述列——與看板討論卡同構，符合 LANGUAGE.md「討論識別錨點以 slug 直出」受控例外；封存變更卡保持 name 為標題、新增描述列，內容為封存 proposal.md 的 Why 首句。
- **已封存清單 payload 疊加呈現輔助欄位**：Rust 端封存清單為每個封存變更項疊加 Why 首句摘錄與建立日期（自封存目錄的 proposal.md 與 metadata 讀出，比照規格清單 purposeExcerpt 先例）；不可讀／缺席時欄位缺席、清單照常回傳。
- **已封存抽屜標頭補齊**：標題複製鈕（封存變更複製 dated name、封存討論複製 slug）＋出身列（建立者、建立日期、封存日期）。不補進度條與動詞動作列——封存是唯讀定格。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`：detail 抽屜互斥需求擴充「開啟時底層落回看板」；已封存頁的卡片 anatomy 改雙行；已封存抽屜檢視需求補標頭複製鈕與出身列。
- `client-protocol`：已封存清單項新增選填的 Why 首句摘錄與建立日期欄位。

## Impact

- Affected specs: `desktop-app`（已封存項目以抽屜檢視、已封存頁含討論節、detail 抽屜互斥）、`client-protocol`（已封存清單欄位）
- Affected code:
  - Modified: apps/desktop/src/store.ts（openDetail／openDiscussion 落頁）、apps/desktop/core/src/query.rs（封存清單疊加欄位）、packages/ui/src/adapter.ts（ArchivedItem 型別）、packages/ui/src/components/ArchivedList.tsx（卡片雙行）、packages/ui/src/components/ArchivedDrawer.tsx（標頭複製鈕＋出身列）、apps/desktop/src/App.tsx（props 接線，如需）
  - New: （無）
  - Removed: （無）

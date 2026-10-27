## Why

變更詳情抽屜的 header 資訊逐版增生——建立者、產生工具、相對時間、任務數、開工資訊、審查狀態、來源討論、同源變更、進度條、動作列全數平鋪堆疊,無資訊分層;其中來源討論標記以 topic 全句直出,長短不定使同一元件呈現兩種排版(短句與標籤同行、長句各自佔滿一行,左緣參差)。前案 drawer-source-chip-overflow 的截斷止痛已證明「不動籤內容」治不了病根。討論 change-drawer-header-redesign 定案:header 重設計為四層固定結構,來源討論籤改以 slug 直出。

目標使用者是透過 AI 代理跑 SDD 的開發者、PO 與 PM;使用情境為自看板開啟變更詳情抽屜檢視變更的狀態與出身脈絡——這是 propose → apply → archive 全程共用的檢視面。

## What Changes

- 變更詳情抽屜(RichDetailDrawer)header 重構為四層固定結構:標題列(名稱+複製鈕)/狀態列(進度條+百分比+審查章)/出身列(單行)/動作列。
- 來源討論籤改顯示討論 slug,topic 降為滑鼠停留提示;延伸 openspec/LANGUAGE.md「slug 直出」明文例外,適用範圍擴充至變更詳情抽屜與已封存抽屜的來源討論籤及其溢出浮層(隨本變更記錄於 LANGUAGE.md)。
- 出身列恆定單行:頭像+名字(完整 email 收進滑鼠提示)+產生工具+建立時間+開工資訊+「來自」slug 籤+「同源」籤;溢出收「+N」數字籤,點擊以 shadcn Popover 浮層列出其餘可點籤(點擊仍可跳至對應討論/變更);+N 切點採固定顆數上限,非量測式,確保同一變更在任何視窗寬度長相一致。
- 「N/N 任務」計數自 header 移除——任務分頁徽章與進度條已承載同一資訊。
- 審查資訊列自 metadata 列升至狀態列(與進度條同列),狀態詞、蓋章時間與審查者內容不變。
- 已封存抽屜(ArchivedDrawer)的來源討論標記比照 slug 直出與同一溢出規則。
- 新增 shadcn Popover 原語(packages/ui/src/components/ui/popover.tsx),底層相依 @radix-ui/react-popover,與既有 ui/ 原語同源。

## Non-Goals

- 看板卡片(ChangeCard)的來源討論呈現(單一圖示+Tooltip 列全部)維持不變。
- 討論抽屜(DiscussionDrawer)、規格抽屜(SpecDrawer)與各分頁內文不在範圍。
- 不動引擎與 adapter 資料側——sourceDiscussions 已同時攜帶 slug 與 topic,Rust core、CLI、server API 零變化。
- 不改 Speclink Server Web Console——server-web 雖相依 @speclink/ui,但未使用本次改動的兩個抽屜元件。
- 不引入 Popover 以外的新元件原語。
- 討論已排除的方案不再考慮:topic 全文+更強截斷、出身資訊收合區塊、固定拆兩行、流式折行、+N 原地展開、hover tooltip 列溢出清單(不可點,斷跳轉路徑)。

## Capabilities

### New Capabilities

(無)

### Modified Capabilities

- `desktop-app`: 變更詳情抽屜與已封存抽屜的 header 呈現規則變更——四層固定結構、來源討論籤 slug 直出、出身列恆定單行與 +N 浮層溢出、審查資訊列移至狀態列、任務數自 header 移除。

## Impact

- Affected specs: `desktop-app`
- Affected code:
  - New: packages/ui/src/components/ui/popover.tsx
  - Modified: packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedDrawer.tsx、packages/ui/src/components/SourceDiscussionChip.tsx、packages/ui/src/i18n.tsx、packages/ui/package.json、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/archivedDrawer.test.tsx、packages/ui/src/__tests__/discussionDrawer.test.tsx、apps/desktop/src/__tests__/App.test.tsx、openspec/LANGUAGE.md
  - Removed: (無)
- 影響的 app/套件:packages/ui(共用 UI 套件,改動本體)、apps/desktop(消費端,隨套件更新,無自身程式碼改動)。
- 相依變化:新增 @radix-ui/react-popover(shadcn Popover 底層)。
- 相容性影響:純呈現層改動;CLI 人眼輸出與 --json 皆零變化,不破壞回歸對照,無使用者遷移需求。

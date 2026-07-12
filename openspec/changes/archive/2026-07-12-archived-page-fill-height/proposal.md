## Why

已封存頁與規格頁的換頁控制列排在整頁捲動的文件流末端，每頁 20 張卡片必然把它推出視窗外——透過桌面 app 瀏覽 SDD 專案的使用者每次換頁都得先捲到清單底部才按得到上一頁／下一頁（來源討論 archived-page-fill-height，附截圖佐證）。看板頁已採「填滿高度＋欄內捲動」版面，清單頁沿用同一策略可同時解決此問題並統一全 app 的捲動模型。

## What Changes

- 已封存頁（ArchivedList）改為填滿高度的 flex 直欄版面：搜尋框與「變更／討論」子頁籤固定頂部、卡片清單於內部容器捲動、換頁控制列（ListPager）固定底部常駐可見，不捲動即可換頁。
- 規格頁（SpecList）比照辦理——同為換頁控制列排清單末的結構，一併修正以維持兩清單頁行為一致。
- 桌面殼主內容區對已封存頁與規格頁改用 overflow-hidden（原僅看板頁），使內部捲動容器高度受視窗約束。
- 換頁後「捲回清單頂部」由整頁 scrollIntoView 改為重置內部捲動容器的捲動位置。
- 已封存頁兩個子頁籤維持各自獨立的換頁控制列與頁碼（現狀不變）。

## Non-Goals

- 不改每頁筆數（20）、排序規則、搜尋過濾與頁碼鉗制等既有換頁語意——僅動版面與捲動行為。
- 不採 position: sticky 底部換頁控制列——清單短時控制列不沉底，且與看板捲動策略分歧（討論中已排除）。
- 不合併兩個子頁籤為跨頁籤共用底欄——需提升頁碼狀態並依作用中頁籤切換，複雜度增加而無收益（討論中已排除）。
- 不涉及 speclink-core / speclink-cli 兩個 crate——純桌面前端呈現變更，CLI 人眼與 --json 輸出無任何變動，既有回歸對照不受影響。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「清單最新在前與換頁瀏覽」需求——換頁控制列由「清單末端、隨內容捲出視窗」改為「頁面底部常駐可見」；清單頁版面填滿視窗高度、卡片清單於內部容器捲動；換頁捲回頂部改為內部捲動容器歸位。排序、每頁筆數、鉗制等其餘語意不變。

## Impact

- Affected specs: `desktop-app`（修改「清單最新在前與換頁瀏覽」需求）
- Affected code:
  - Modified: packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/SpecList.tsx、apps/desktop/src/App.tsx、packages/ui/src/__tests__/archivedList.test.tsx、packages/ui/src/__tests__/specList.test.tsx、apps/desktop/src/__tests__/App.test.tsx
  - New: (none)
  - Removed: (none)

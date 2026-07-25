## Why

桌面 app 的變更詳情抽屜與已封存抽屜，在標頭列出「來源討論」時把整段討論 topic 直接塞進一枚圓角標記。討論 topic 是自由文字、可長達整句話，該標記沒有寬度上限也不截斷，於是單一標記本身就比抽屜還寬。標記所在容器雖有換行設定，但換行只能在項目之間斷開，救不了「單一項目寬於容器」的情況；抽屜面板本身設定了垂直捲動，依 CSS 規則水平方向連帶變成自動捲動——結果是整個抽屜出現水平捲軸，標頭、動作列與內文全部被推得可左右滑動。

對透過 AI 代理執行 SDD 的開發者、PO 與 PM 而言，這讓「從變更跳回它的來源討論」這個日常動作變成一次版面事故：抽屜內每個區塊都要左右捲才看得完。此缺陷在討論 topic 較長時必然出現，而 topic 長句正是討論記錄的常態。

## What Changes

- 詳情抽屜與已封存抽屜的來源討論標記 SHALL 受抽屜可用寬度約束：過長的 topic 以截斷呈現，並以原生提示保留全文。
- 抽屜面板本身不再於水平方向產生捲軸，標頭、動作列與內文一律維持在可視寬度內。
- 兩處抽屜採同一種處理方式，避免只修一處、另一處保留相同缺陷。

## Non-Goals

- 不改變來源討論標記的資料來源、數量、排序或點擊後開啟討論的行為。
- 不改變抽屜寬度、全螢幕切換與其餘標頭元素的既有版面。
- 不調整看板卡片、討論欄位或已封存清單上既已截斷的 topic 呈現。
- 不修改 Speclink Server Web Console 的任何檔案——後台重新設計由 `admin-console-redesign` 處理。
- 不引入新的元件庫原語或新相依。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增一條抽屜版面約束——來源討論標記不得撐寬抽屜、抽屜不得產生水平捲軸。既有的來源討論多值呈現行為（代表徽章、列出全部、互跳、同源判定）不變。

## Impact

### 目標使用者與情境

透過 AI 代理執行 SDD 的開發者、PO 與 PM；情境為在桌面 app 看板上開啟變更詳情抽屜或已封存抽屜、循來源討論標記跳回討論記錄。對應 Speclink workflow 的 propose 之後、archive 之前的日常檢視，不影響任何 SDD 動詞的執行。

### 影響的 crate

無。本變更只動前端共用元件庫 `packages/ui`，不涉及 `speclink-core`、`speclink-cli` 或其他 crate。

### 相容性影響

- **CLI**：無變更。人眼輸出與 `--json` 皆不動，回歸對照不受影響。
- **資料與 meta 欄位**：無變更。`from_discussion` 的解析與呈現語意完全不動。
- **使用者可見行為**：來源討論標記在 topic 過長時由完整呈現改為截斷加提示；短 topic 的呈現與現況一致。

### 設定與技能

- 不新增或變更 `.speclink.yaml` 與 `openspec/config.yaml` 的任何欄位。
- 不影響 claude 或 codex 的技能內容與注入區塊。

### Affected code

- Modified:
  - `packages/ui/src/components/RichDetailDrawer.tsx`
  - `packages/ui/src/components/ArchivedDrawer.tsx`
  - `packages/ui/src/components/ui/sheet.tsx`
  - `packages/ui/src/__tests__/richDrawer.test.tsx`
  - `packages/ui/src/__tests__/archivedDrawer.test.tsx`
- New:
  - `packages/ui/src/components/SourceDiscussionChip.tsx`

規格要求截斷約束「SHALL 同時套用於變更詳情抽屜與已封存抽屜，SHALL NOT 只在其中一處成立」。兩處抽屜原本各有一份相同的標記樣式，容易再度分歧，因此把標記收斂為單一共用元件而非在兩處各改一次；該元件不對外匯出，僅供這兩個抽屜使用。

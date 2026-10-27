## Context

- **現況**：規格抽屜（packages/ui 的 SpecDrawer）標頭只有標題。溯源資訊由前端的 parseTraceSources 從正典全文的 `@trace` 註解抽出來源變更名，渲染為內文底部一行灰字「來源變更：a、b」，不可點。已封存抽屜（ArchivedDrawer）的變更型別標頭有標題列（名稱＋複製鈕）與出身列（建立者、日期、「來自」＋討論籤），討論籤由共用的 SourceChipRow 元件呈現（首籤直出、其餘收「+N」浮層）；討論型別的出身列只有建立者與建立日期，衍生變更（promotedTo）只在封存討論卡以數字徽章呈現，抽屜內無名單、無跳轉。變更詳情抽屜（RichDetailDrawer）的出身列同樣用 SourceChipRow 呈現「來自」與「同源」兩列。
- **資料**：封存變更清單（ArchivedItem：datedName、date、name、fromDiscussions）與封存討論清單（含 promotedTo）在開工作區時由 store 全量載入（listArchived 無分頁；remote 與 tauri 兩個資料源皆實作）。抽屜不需要任何新查詢。
- **引擎**：`speclink trace` 已能輸出整條「規格→變更→討論」鏈，但 desktop 不經它——變更名到封存目錄的對應由前端清單完成，local 與 remote 同一套行為，server 側也不必新增端點。
- **正典約束**：detail 抽屜互斥（規格抽屜開封存抽屜屬合法轉移，先開者關閉）；規格抽屜與封存抽屜開啟時底層頁面不切換，變更詳情抽屜開啟時底層落回看板；抽屜標頭出身列恆定單行、不撐寬抽屜（SourceChipRow 的「首籤＋N」即此保證）。
- **相關方**：透過 AI 代理跑 SDD 的開發者、PO 與 PM。來源討論 `spec-drawer-trace-links`。

## Goals / Non-Goals

**Goals:**

- 三個唯讀抽屜（規格、封存變更、封存討論）的標頭文法一致：標題列＋出身列，來源連結一律是同一個籤元件。
- 「手冊出處 → 規格抽屜 → 封存變更抽屜 → 封存討論抽屜 → 封存變更抽屜」鏈的每一跳都可點。
- 零引擎改動、零新查詢；local 與 remote 模式同行為。

**Non-Goals:**

- 逐條需求旁就地標「來自哪個變更」（trace JSON 的 requirements[].source 已可支撐，留作下一步）。
- 規格抽屜內嵌 `speclink trace` 的整棵樹。
- 已封存討論抽屜加第四區段「衍生變更」（子變更幾乎全為已封存，「現況」欄無資訊量，且與正典的三區段字面衝突）。
- 手冊取材加入討論（討論已裁定手冊維持只讀正式規格）。
- 抽屜返回堆疊（互斥規則維持現狀）。
- 同名多份封存目錄的解析規則（目前 199 份封存變更無重名，正典沉默，不預作）。
- 規格抽屜出身列加「最近更新」日期。
- 活討論抽屜的「衍生變更」分頁不動。

## Decisions

### D1 溯源籤搬進規格抽屜標頭並移除內文底部的 footer

規格抽屜標頭改為兩層：標題列（capability 名＋複製名稱鈕，複製後短暫已複製回饋，與規格卡的複製鈕同款）與出身列（前綴「來自」＋ SourceChipRow）。無狀態列、無動作列——正典唯讀，與已封存抽屜「無進度條與動詞動作列」同理。內文底部的「來源變更：」文字行整段移除，不保留任何替代文字。

- 替代方案「保留 footer 只加連結」：與另外兩個抽屜的標頭文法不一致，正是使用者指出的問題，出局。
- 替代方案「加狀態列」：規格沒有進度或站章，出局。

### D2 封存清單由 host 傳入並在抽屜內解析與排序

SpecDrawer 新增兩個 prop：`archivedChanges`（ArchivedItem 陣列）與 `onOpenArchivedChange(datedName)`。抽屜以既有的 parseTraceSources(doc) 取得來源變更名（去重、依文件首次出現序），再對 archivedChanges 以 name 比對：

- 命中 → 籤項 `{ slug: name, topic: 封存日期 }`，可點，點擊回呼帶該項的 datedName。
- 未命中 → 籤項 `{ slug: name, topic: 「無封存記錄」, disabled: true }`。

排序：命中者依封存日期升冪（同日依文件首次出現序），未命中者排在最後。第一顆籤因此是最早封存的變更，即此 capability 的出身，與變更詳情抽屜「首籤為出身討論」對稱。

- 替代方案「走 `speclink trace`」：需新增 desktop 查詢層，server 無 trace 端點，remote 模式做不到，出局。
- 替代方案「host 預先解析」：文件由抽屜載入，host 沒有全文，出局。此做法與活討論抽屜接收 changes／archivedChanges 清單的既有型式相同。

### D3 來源連結籤的不可點狀態

SourceLinkItem 補選填欄位 `disabled`。SourceDiscussionChip 在 disabled 時：灰底灰字、無 hover 樣式、無 onClick、帶 `aria-disabled="true"`；Tooltip 仍顯示 slug 與副標。溢出浮層內的 disabled 項同樣不可點且不關閉浮層。此一套機制同時供規格抽屜（無封存記錄的來源變更）與已封存討論抽屜（已刪除的子變更）使用。

- 替代方案「過濾掉未命中的名稱」：籤會與正典全文的 `@trace` 對不上，出局。
- 替代方案「沿用看板討論卡 chip 的刪除線樣式」：那是看板卡片的語意，抽屜出身列統一灰化，出局。

### D4 已封存討論抽屜的衍生列與三態跳轉

ArchivedDrawer 新增兩個 prop：`promotedChanges`（SourceLinkItem 陣列，含 disabled）與 `onOpenPromotedChange(name)`。目標為討論型別且清單非空時，於出身列之下渲染 SourceChipRow（前綴「衍生」），位置與變更型別的「來自」列相同；變更型別不渲染此列。

host（apps/desktop 的 App）自封存討論清單項的 promotedTo 派生三態，順序沿 promotedTo 原序（轉出順序，出身在前，與活討論抽屜的分頁一致）：

| 子變更所在 | 副標 | 可點 | 點擊 |
| --- | --- | --- | --- |
| 封存清單命中 | 封存日期 | 是 | openArchived({ kind: "change", datedName }) |
| 活躍變更清單命中 | 看板階段詞（重用 discussionChipStage） | 是 | openDetail(name)，底層依正典落回看板 |
| 兩者皆無 | 「無封存記錄」 | 否 | 無 |

- 替代方案「比照活討論抽屜加第四區段」：見 Non-Goals。

### D5 標籤字與詞彙

- 規格抽屜出身列前綴「來自」：新增 i18n key `sdrawer.fromChanges`（zh「來自」／en「From」），文字與 `rdrawer.fromDiscussion` 相同但語意獨立、各自可改。
- 已封存討論抽屜衍生列前綴「衍生」：新增 key `adrawer.promotedTo`（zh「衍生」／en「Derived」）。
- 不可點籤副標：新增 key `rdrawer.noArchiveRecord`（zh「無封存記錄」／en「No archive record」）。
- 移除 `specs.sourceChanges` 與 `specs.sourceSep` 兩個 key（footer 隨之消失，無其他呼叫端）。
- `openspec/LANGUAGE.md` 的「衍生變更」詞條 definition 補一句：抽屜出身列的標籤縮寫為「衍生」，與「來自」「同源」同為兩字關係詞。不新增詞條、不動 avoid 欄（避免詞彙守門掃描面波及）。

## Implementation Contract

**Behavior（使用者可觀察）**

1. 開啟 capability X 的規格抽屜：標頭第一行為 X 與複製名稱鈕，點鈕把 X 寫入剪貼簿並短暫顯示已複製回饋；第二行為「來自」＋第一顆籤（最早封存的來源變更），來源多於一個時緊接「+N」籤，點「+N」開浮層列出其餘全部（主行變更名、副標封存日期）；點任一可點籤（含浮層項）→ 規格抽屜關閉、該封存變更的唯讀抽屜開啟、底層頁面維持原頁（規格頁或手冊頁）。正典全文無 `@trace` 來源時第二行缺席。內文底部不再出現「來源變更：」文字。
2. 來源變更名在封存清單中無對應時，該籤灰化不可點，Tooltip 副標「無封存記錄」；排序在所有可點籤之後。
3. 開啟一筆 promotedTo 非空的封存討論抽屜：出身列之下有「衍生」列，第一顆籤＋「+N」；三態依 D4 表。promotedTo 為空時該列缺席。封存變更抽屜不顯示「衍生」列。

**Interface / data shape**

- SpecDrawer props 新增：`archivedChanges: ArchivedItem[]`、`onOpenArchivedChange?: (datedName: string) => void`。
- ArchivedDrawer props 新增：`promotedChanges?: SourceLinkItem[]`、`onOpenPromotedChange?: (name: string) => void`。
- SourceLinkItem 新增：`disabled?: boolean`。
- i18n 新增 key：`sdrawer.fromChanges`、`adrawer.promotedTo`、`rdrawer.noArchiveRecord`；移除 `specs.sourceChanges`、`specs.sourceSep`。
- 無 CLI、`--json`、IPC 或設定變更。

**Failure modes**

- archivedChanges 為空（例如 remote 會話缺 listArchived 能力）：所有來源變更籤呈不可點「無封存記錄」，不報錯、不崩。
- 規格文件載入失敗：既有骨架與空態行為不變，出身列缺席。
- 封存討論的 promotedTo 指向已刪除的變更：灰籤，不報錯。

**Acceptance criteria**

- packages/ui 測試：specDrawer.test.tsx（標頭籤、排序、+N 浮層、點擊回呼、不可點、無 @trace 缺席、footer 文字不存在、複製鈕）、archivedDrawer.test.tsx（衍生列三態、空清單缺席、變更型別無此列）、sourceDiscussionChip 的 disabled 行為。
- apps/desktop 測試：App.test.tsx（規格抽屜點籤→封存抽屜開啟且 boardView 不變；封存討論抽屜點衍生籤→封存變更抽屜或詳情抽屜）。
- 指令：`npm test -w packages/ui`、`npm test -w apps/desktop`、`node --test scripts/*.test.mjs`。

**Scope boundaries**

- In：packages/ui 的 SpecDrawer、ArchivedDrawer、SourceDiscussionChip、i18n 與三個測試檔；apps/desktop 的 App 接線與測試；openspec/LANGUAGE.md 一句用法註記；desktop-app 兩條 MODIFIED 需求。
- Out：Non-Goals 全部；任何 Rust crate；CLI 與 server；活討論抽屜；看板卡片。

## Risks / Trade-offs

- [既有 specDrawer 測試有四處斷言 footer 字面「來源變更：」] → 同批改為標頭籤斷言，不保留舊斷言。
- [「來自」文字在兩個 i18n key 重複] → 語意獨立、各自可改；接受重複。
- [同名多份封存目錄] → 目前無；取 name 首個命中；記入 Non-Goals，不預作規則。
- [remote 會話封存清單為空] → 全部灰籤降級，不崩。
- [新文案觸發詞彙守門] → 「來自」「衍生」「無封存記錄」不含 LANGUAGE.md 的 avoid 詞；`node --test scripts/*.test.mjs` 守門。
- [golden 與 CLI 測試] → 純前端變更，不動任何 CLI 輸出，golden 不受影響。
- [跨平台] → 純前端 DOM 與 i18n，無路徑或換行假設。

## Migration Plan

無資料遷移、無設定變更。回滾即還原前端程式碼與 i18n。

## Open Questions

無。

## Why

桌面 app 的失敗訊息在最需要被看到的時候看不見。視窗頂列狀態列（`store.verbResult`）是全 app 唯一「回饋離觸發點很遠」的面，而抽屜的遮罩層是全視窗覆蓋、頂列位於其下——「封存」與「刪除」正是從抽屜動作列觸發的兩個操作，於是：刪除成功時抽屜關閉、訊息看得到（但這條最不需要，卡片消失本身即回饋）；刪除與封存失敗時抽屜不關、訊息被遮罩蓋住（但這條最需要）。可見性完全相反，「失敗時 SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗」在此形同虛設。

該面另有兩項缺陷：一是無生命週期——`verbResult` 只在初始狀態設為空，此後無逾時、無關閉鈕，切分頁／切頁／關抽屜皆不清，只被下一次全域操作覆蓋，否則一路殘留到關閉 app（使用者即因刪除後「已刪除」永久掛在頂列而提出此議題）；二是封存成功訊息混用英文動詞且未走 i18n，抵觸 UI 文字繁體中文硬編與工程詞禁令。

**目標使用者**：透過 AI 代理跑 SDD 的開發者 / PO / PM。**使用情境**：於桌面 app 看板執行全域操作（刪除變更、封存變更、封存討論、拖排卡片、開啟專案、初始化專案）——對應 propose 之後、apply／archive 期間的日常看板操作，不限定單一 workflow 階段。

承討論 `desktop-feedback-surface` 的結論（含其 Round 3 對 toast 實作路徑的反轉）。

## What Changes

改採「畫面說了沒有」判準：畫面已表達的結果靜默，畫面未表達的才浮出 toast。

- **移除視窗頂列狀態列**：刪除 `store.verbResult` 狀態欄位與其於頂欄的渲染。**BREAKING**（僅使用者可見行為，無 API／CLI 契約變動）。
- **3 條成功寫入改為靜默**：刪除變更成功（卡片自看板消失即回饋）、封存討論成功、封存變更成功（卡片消失且側欄「已封存」計數遞增）。封存成功訊息一併移除，順帶解消其混用英文動詞、未走 i18n 的規範違例——封存後的日期化名稱於「已封存」頁可見，非唯一出口。
- **6 條失敗寫入改走 toast**：刪除變更失敗、封存討論失敗、封存變更失敗、拖排寫回失敗、開啟專案失敗、初始化專案失敗。
- **引入 sonner 作為 toast 呈現原語**：走 shadcn registry 的 `@shadcn/sonner` 路徑，於前端共用元件庫的設計系統原語層新增 `Toaster`，並拆除該 registry wrapper 的 `next-themes` 相依——sonner 的 `theme="system"` 直接對接桌面 app 既有的 `prefers-color-scheme` 深色模式。sonner 的浮層層級遠高於抽屜與確認框的遮罩層，此即修掉「訊息被抽屜遮罩蓋住」的關鍵；逾時、關閉鈕與 hover 暫停計時皆由 sonner 提供。新增依賴 `sonner`（零執行期相依、peer 僅 react／react-dom）。
- **store 不再保留失敗狀態**：sonner 的 toast API 為 module-level singleton、可於 React 之外呼叫，因此 `verbResult` 直接刪除、不以任何狀態欄位取代——store 於失敗路徑直接呼叫 sonner 發出訊息。
- **封存成功時關閉詳情抽屜**：封存動詞目前不清除詳情抽屜的選定狀態。若成功靜默而抽屜仍開著蓋住看板，「畫面說了」的前提即不成立、使用者會感覺沒反應。刪除成功已有此行為，封存漏了——此為判準成立的必要條件，非附帶優化。
- **開啟專案失敗與初始化專案失敗補上主詞前綴**：此二者目前寫入裸錯誤字串、無主詞，與其餘 4 條失敗訊息格式不一致；改走 toast 後若無主詞，使用者無從得知何者失敗。

## Non-Goals

- **不改動其他既有回饋面**：任務勾選寫回失敗（詳情抽屜內的就地錯誤）、專案分頁錯誤態、伺服器連線狀態、設定頁「已儲存 ✓」一律維持現狀。此四者皆有可見錨點且不被遮罩遮蔽，在「畫面說了沒有」判準下無需改動——本刀不重構沒壞的東西。
- **toast 不做堆疊**：同時僅呈現一則失敗訊息，新訊息取代前一則。失敗訊息不會併發，堆疊佇列屬臆測性設計——以 sonner 的固定 toast id 與單槽設定達成，不自行實作佇列。
- **不改動確認框的確認流程行為**：刪除與封存的確認對話框語意不變。
- **拖排成功維持靜默**：既有刻意行為（`board-card-order` 已明寫「拖排失敗」時才呈現錯誤），不改。
- **不改動 board-card-order 規格**：其「寫回失敗時 SHALL 以單行錯誤訊息呈現並刷新回磁碟現況」未指定呈現面，toast 仍滿足該需求，無 delta。
- **不觸及任何 Rust crate**：speclink-core 與 speclink-cli 皆不受影響。無 CLI 子指令、旗標、stdin 或 exit code 變動，人眼輸出與 `--json` 契約不變，回歸對照不受影響。
- **無設定欄位變動**：`.speclink.yaml` 與 `openspec/config.yaml` 皆不涉及。
- **無技能或注入區塊變動**：不影響 claude 或 codex 的技能檔與 marker。
- **不引入 `next-themes`**：shadcn 的 sonner wrapper 為 Next.js 而用它取得主題，本專案深色模式由 OS 的 `prefers-color-scheme` 驅動，改以 sonner 的 `theme="system"` 對接。
- **拒絕的做法：純 React 自建 toast**——輸在與共用元件庫 `ui` 原語層的 shadcn 慣例不一致（會是該層唯一非 shadcn 出身的原語），且浮層層級、hover 暫停與無障礙須自理；其唯一優勢「零依賴」在 sonner 同為零執行期依賴的事實下不成立。
- **拒絕的做法：`@radix-ui/react-toast`**——輸在 shadcn registry 已下架 toast 元件，該套件不再是 shadcn 路徑。
- **拒絕的做法：頂列加逾時自動消失**——只治永久殘留，留著「被抽屜遮罩蓋住」與「離觸發點遠」，且救不了拖排失敗與開啟專案失敗兩個無錨點缺口。
- **拒絕的做法：全部失敗都貼回觸發點、不引 toast**——拖排失敗（卡片跳回原位不解釋原因，卡片非錯誤歸屬對象）與開啟專案失敗（原生選檔對話框已關閉、分頁尚未建立）無錨點可貼，硬推會逼出第二個回饋面、反而更不一致。
- **拒絕的做法：所有操作皆 toast（含拖排成功）**——拖排的回饋是卡片跟著手到新位置，文字加不了資訊；重複噪音會侵蝕 toast 面的可信度，使真正的失敗訊息被埋沒，等於換一種方式回到「看不見的回饋面」。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「桌面 app 提供動詞操作面」需求改寫——視窗頂列狀態列不再存在；失敗訊息改由 toast 呈現且其層級 SHALL 高於抽屜與確認框的遮罩層；全域操作成功時 SHALL NOT 呈現文字訊息；封存成功 SHALL 關閉詳情抽屜。

## Impact

- Affected specs: `desktop-app`
- Affected code:
  - New:
    - `packages/ui/src/components/ui/sonner.tsx`
    - `packages/ui/src/__tests__/sonner.test.tsx`
  - Modified:
    - `packages/ui/package.json`
    - `packages/ui/src/index.ts`
    - `packages/ui/src/i18n.tsx`
    - `apps/desktop/package.json`
    - `apps/desktop/src/store.ts`
    - `apps/desktop/src/App.tsx`
    - `apps/desktop/src/i18n/messages.ts`
    - `apps/desktop/src/__tests__/store.test.ts`
    - `apps/desktop/src/__tests__/workspace.test.ts`
    - `apps/desktop/src/__tests__/App.test.tsx`
  - Removed: (none)
- 相容性影響：僅桌面 app 的使用者可見行為改變（頂列狀態列消失、失敗改以 toast 浮現、成功不再有文字訊息）。新增前端執行期依賴 `sonner`（`packages/ui` 與 `apps/desktop` 各一筆——前者提供 `Toaster`、後者於 store 發出訊息）。無 CLI、`--json`、core API 或磁碟格式變動，既有使用者無需遷移。

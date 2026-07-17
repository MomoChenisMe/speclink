## Context

桌面 app 目前以 store 的 `verbResult` 字串欄位承載全域操作結果，由頂欄渲染成一行等寬灰字。該欄位有 9 個寫入點（3 成功 / 6 失敗），且全檔只在初始狀態被設為空——無逾時、無關閉鈕，只會被下一次全域操作覆蓋。

抽屜與確認框的遮罩層皆為全視窗覆蓋、層級 `z-50`；頂欄未設層級，因此躺在遮罩之下。而「封存」與「刪除」正是詳情抽屜動作列上的按鈕——失敗時抽屜不關，錯誤訊息寫入被遮罩蓋住的頂欄，等於靜默。

前端既有回饋面除頂列外皆為「貼在觸發點旁」：任務勾選寫回失敗呈於詳情抽屜內、專案分頁錯誤呈於分頁上、連線狀態呈於連線列、設定頁「已儲存 ✓」呈於各區段儲存鈕旁、複製回饋就地呈現並於 1200 毫秒後消失。頂列是唯一的例外。

共用元件庫的 `ui` 原語層以註解明示「shadcn/ui 設計系統原語」，現有 button、card、badge、alert-dialog、sheet、checkbox、select、tabs、textarea、input、tooltip 全數出自 shadcn，其 Radix 基底的原語各自帶對應的 `@radix-ui/*` 套件。

**crate 邊界**：本變更完全不落在任何 crate——`speclink-core` 與 `speclink-cli` 皆不受影響，無流程邏輯、無 ANSI、無儲存假設變動。改動範圍純屬前端 TypeScript（共用元件庫 `packages/ui` 與桌面殼層 `apps/desktop`），透過既有 `SpeclinkDataSource` 介面取得資料，不新增 IPC 動詞。

承討論 `desktop-feedback-surface` 的結論（含 Round 3 對 toast 實作路徑的反轉）與本變更的提案。

## Goals / Non-Goals

**Goals:**

- 失敗訊息在抽屜或確認框開啟時仍必然可見——這是本刀的核心修復，其餘皆為推論結果。
- 回饋具明確生命週期：出現、可讀、消失，不殘留。
- 落實「畫面說了沒有」判準：畫面已表達的結果靜默，畫面未表達的才浮出。
- 全域操作的失敗訊息格式一致（皆帶主詞、皆走 i18n）。
- toast 原語與 `ui` 原語層的 shadcn 慣例一致。

**Non-Goals:**

- 不改動任務勾選寫回失敗、專案分頁錯誤態、連線狀態、設定頁儲存回饋——四者皆有可見且不被遮蔽的錨點，本刀不重構沒壞的東西。
- 不自行實作 toast 佇列、堆疊、手勢或層級計算——全數交由 sonner。
- 不引入 `next-themes`。
- 不改動確認對話框的確認流程語意。
- 不改動拖排成功靜默的既有行為。
- 不觸及 CLI、`--json` 契約、core API 或磁碟格式。

## Decisions

### D1: store 直接呼叫 sonner 的 toast API，不保留失敗狀態

6 個失敗寫入點全部位於 store 的動作函式內（刪除、封存討論、封存變更、拖排、開啟專案、初始化專案），皆非 React 元件。sonner 的 toast API 是 module-level singleton、可於 React 之外呼叫，因此 store 於失敗路徑直接呼叫它發出訊息；`verbResult` 直接刪除，**不以任何狀態欄位取代**，也不需要清除動作。淨結果是 store 的狀態面積比現況更小。

store 因此會 import sonner。這是本決策的代價：store 目前無執行期 UI import（其對共用元件庫的 import 是純型別、執行期抹除）。可接受的理由是 store 已 import i18n 執行期模組來組出使用者可見的訊息字串——它早已承擔「產出給人看的文字」這件事，發出該文字是同一方向的延伸，而非新的耦合方向。

- **替代方案：store 保留 `failure: string | null`，由 App 以 effect 轉呼 sonner**——否決。雙重記帳（sonner 內部已是 toast 的狀態來源，再加一份即兩個真相）；且相同訊息連續兩次失敗時 effect 的相依比對不會重新觸發，得再補一個遞增 id 才能修，複雜度純屬自找。

### D2: toast 層級由 sonner 提供，不自訂 z-index

sonner 的 toaster 容器自帶遠高於本專案全部 `z-50` 遮罩層的層級，掛載於 `document.body` 的 portal。**這是本刀唯一真正修掉「訊息被遮罩蓋住」的機制**，且為久經沙場的實作而非自行計算的層級。

toast 疊在抽屜上方是刻意的：失敗發生時使用者正需要看到它，讓路給抽屜等於重演今天的 bug。

- **替代方案：提高頂欄的層級讓它浮上遮罩**——否決。頂欄浮在遮罩之上視覺突兀（遮罩的用意就是壓暗背景），且完全不解決「回饋離觸發點遠」與「無生命週期」，只是把看不見換成看得見卻依然被忽略。

### D3: 逾時、關閉鈕與單槽取代語意由 sonner 設定提供

`Toaster` 以 `duration={6000}`、`closeButton`、`visibleToasts={1}` 設定；失敗訊息一律以同一個固定 toast id 發出，使新訊息更新既有 toast（取代前一則並重置其逾時）而非堆疊。hover 暫停計時為 sonner 內建行為，不另付成本。

6000 毫秒高於既有複製回饋的 1200 毫秒，因為錯誤訊息含路徑與 core 錯誤、閱讀量大得多。不自動消失是不可接受的：使用者提出此議題的起因正是「訊息不消失」——若 toast 常駐至手動關閉，同一個抱怨會原封不動再來一次，只是換了個位置。

- **替代方案：不設固定 id，改以 `visibleToasts={1}` 限制可見數**——否決。那是佇列語意（舊訊息排隊等待）而非取代語意，與「同時僅一則、新訊息取代前一則」的規格不符。
- **替代方案：沿用 1200 毫秒**——否決，讀不完一則帶路徑的錯誤訊息。

### D4: toast 採 shadcn registry 的 sonner，拆除其 wrapper 的 next-themes 相依

`ui` 原語層以註解明示為 shadcn 原語且現有 11 個原語全數出自 shadcn；toast 亦取自 registry 以維持該慣例。shadcn 的 `@shadcn/toast`（Radix 基底）已自 registry 下架，`@shadcn/sonner` 是其現行且唯一的 toast 路徑，故不存在「用 Radix toast」這個選項。

該 registry item 宣告的相依為 `sonner` 與 `next-themes`，但 `next-themes` 只是其 wrapper 為 Next.js 取得主題所需——sonner 自身零執行期相依，peer 僅 react／react-dom（含 19）。本專案的深色模式由 `apps/desktop/src/index.css` 的 `@media (prefers-color-scheme: dark)` 驅動（OS 層級，非 class 切換），與 sonner 的 `theme="system"` 語意一致，故 wrapper 改為不接 `next-themes`、直接指定 `theme="system"`。

- **替代方案：純 React `createPortal` 自建**——否決。會是 `ui` 原語層唯一非 shadcn 出身的原語；層級、hover 暫停、無障礙須自理；其唯一優勢「零依賴」在 sonner 同為零執行期依賴的事實下不成立。
- **替代方案：`@radix-ui/react-toast`**——否決，已自 shadcn registry 下架，不再是 shadcn 路徑。

### D5: 封存成功時關閉詳情抽屜

封存動詞成功時一併清除詳情抽屜的選定狀態。刪除成功已有此行為，封存漏了。

此條是 D 系列裡唯一不屬於 toast 機制、卻是判準成立的必要條件：成功之所以能靜默，前提是「畫面說了」——卡片自看板消失、側欄「已封存」計數遞增。若抽屜仍開著蓋住看板，畫面什麼都沒說，使用者會感覺沒反應，而那正是 2026-07-09 已判定過的失敗模式。

注意這與既有的「detail 抽屜互斥」需求無關：互斥規範的是「開啟動作清除其他抽屜」，此處是關閉動作。

- **替代方案：封存成功也給 toast**——否決。畫面已充分表達（卡片消失＋計數遞增），文字是重複；且會讓 toast 面開始承載成功訊息，稀缺性隨即流失。

### D6: 失敗訊息文字由 desktop i18n 組出，sonner 不涉訊息內容

訊息維持既有組法「主詞 · i18n 失敗描述 ✗ core 錯誤」，於 store 內以 desktop 的訊息表組出後交給 sonner；sonner 只負責呈現與生命週期。`Toaster` 自身唯一的文案是關閉鈕的無障礙標籤，以其 `closeButtonAriaLabel` 設定取自共用元件庫的 i18n。

開啟專案失敗與初始化失敗目前寫入裸錯誤字串、無主詞，補上主詞（分別為使用者選定的路徑與目錄），與其餘 4 條對齊。封存失敗新增 i18n 鍵取代目前混用的英文動詞。

- **替代方案：改用 sonner 的 `toast.error(title, { description })` 兩層排版**——否決。6 條訊息的結構完全相同且皆為單行，拆成標題／描述兩層只是把一次字串組裝換成一組參數，無實益。

## Implementation Contract

**Behavior**（使用者可觀察）

- 視窗頂欄不再有任何狀態文字。
- 刪除變更成功、封存變更成功、封存討論成功：無文字訊息；卡片自看板消失（封存另使側欄「已封存」計數遞增）；若操作自詳情抽屜觸發，抽屜關閉。
- 上述三者失敗，以及拖排寫回失敗、開啟專案失敗、初始化專案失敗：浮出 toast，內容為「主詞 · 失敗描述 ✗ core 錯誤」，6 秒後自動消失，期間可按關閉鈕提前關閉，滑鼠停留其上時計時暫停。
- 抽屜或確認框開啟時失敗，toast 仍完整可見、浮於遮罩之上。
- 連續兩次失敗：僅呈現一則 toast，後者取代前者並重置逾時。
- 拖排成功：無文字訊息（既有行為不變）。

**Interface / data shape**

- `packages/ui` 匯出 `Toaster`（`components/ui/sonner.tsx`），為 sonner `Toaster` 的薄包裝，設定 `theme="system"`、`duration={6000}`、`closeButton`、`visibleToasts={1}`、`closeButtonAriaLabel` 取自共用元件庫 i18n 的 `toast.close`。不接 `next-themes`。
- `apps/desktop` store：`verbResult: string | null` 欄位刪除，無替代欄位、無清除動作。六條失敗路徑改為以 sonner 的 error 型 toast 發出訊息，一律帶同一個模組層級的固定 toast id 常數以達成取代語意。
- desktop 訊息表：新增 `store.archiveFailed`、`store.openProjectFailed`、`store.initFailed`；移除 `store.deleted`、`store.discussionArchived`；保留 `store.deleteFailed`、`store.discussionArchiveFailed`、`store.reorderFailed`、`store.tabInvalid`。
- 共用元件庫訊息表：新增 `toast.close`（zh-TW 與 en 各一）。
- 依賴：`packages/ui` 與 `apps/desktop` 的 package.json 各新增 `sonner`（前者用 `Toaster`、後者用 toast API）。

**Failure modes**

- 刻意靜默：三條成功路徑、拖排成功路徑——皆不發出 toast。
- 刻意浮出：六條失敗路徑——皆發出 toast，訊息必含主詞與 core 原始錯誤字串，SHALL NOT 吞掉或改寫 core 錯誤內容。
- 失敗後既有的「刷新回磁碟現況」行為不變，不留未落檔的假象。

**Acceptance criteria**

- store 測試（以 vitest 對 sonner 模組設 mock）：三條成功路徑執行後未發出任何 toast；六條失敗路徑執行後各發出一次 toast，訊息同時包含主詞與 core 錯誤字串，且皆帶同一個固定 id；封存成功後詳情抽屜選定狀態為 null，封存失敗後維持不變。
- `Toaster` 整合測試（不 mock sonner）：掛載後發出一則訊息，訊息可見且帶關閉鈕；以同一 id 再發一則，僅後者可見、前者不在畫面上（取代語意，非佇列）；以假計時器推進 6000 毫秒後訊息消失；點按關閉鈕立即消失。
- **真實視窗驗證（不可省略）**：jsdom 測不出實際堆疊順序，而堆疊正是本刀的核心修復。須以 release app 實測：開啟詳情抽屜 → 觸發一次會失敗的封存或刪除 → 截圖確認 toast 完整浮現於遮罩之上且文字可讀 → 確認 6 秒後自行消失。操作前先確認使用者未在使用螢幕。
- 既有測試全綠：`npm test -w packages/ui`、`npm test -w apps/desktop`；建置通過：`npm run build -w apps/desktop`。

**Scope boundaries**

- **In scope**：store 的 `verbResult` 刪除與六條失敗改發 sonner toast、頂欄狀態文字渲染移除、`Toaster` 原語新增與根層掛載、`sonner` 依賴新增、六條失敗訊息的主詞與 i18n 對齊、封存成功關抽屜、`desktop-app` 規格中頂列狀態列條文的改寫。
- **Out of scope**：任務勾選寫回失敗（維持抽屜內就地呈現）、專案分頁錯誤態、連線狀態、設定頁儲存回饋、複製回饋、確認對話框流程、`board-card-order` 規格（其「以單行錯誤訊息呈現」未指定呈現面，toast 仍滿足，無 delta）、任何 crate 與 CLI 行為、`next-themes` 或任何主題切換機制。

## Risks / Trade-offs

- **[jsdom 測不出 z-index 實際堆疊，而那正是本刀的核心修復]** → 單元測試只能驗元件掛載與行為，必須以真實視窗截圖驗證「toast 浮於抽屜遮罩之上」，此步列為獨立任務、不得以測試綠燈代替。sonner 的層級雖遠高於 `z-50`，但 sonner 與 Radix 的 portal 同掛 `document.body`，實際堆疊仍須眼見為憑。
- **[移除頂列後，若 toast 未正確掛載，開啟專案失敗與初始化失敗將完全靜默——比現況更糟]** → 此二條的 store 測試須先寫並見紅，再移除 `verbResult`；TDD 紅→綠順序在此不是形式要求，而是防止把「看不見」升級成「不存在」。
- **[store 因此 import sonner，狀態層直接相依呈現庫]** → 已於 D1 權衡並接受；緩解為 store 只呼叫發出訊息這一個動作、不涉排版與樣式，且該相依可由測試以 mock 隔離，store 的其餘邏輯仍可獨立測試。
- **[新增執行期依賴 sonner]** → sonner 零執行期相依、peer 僅 react／react-dom，非引入新體系；且取自 shadcn registry 的現行路徑，與 `ui` 原語層既有慣例一致。
- **[6 秒逾時可能讓使用者錯過錯誤訊息]** → sonner 內建 hover 暫停計時；失敗後畫面已刷新回磁碟現況、操作可重試並再次觸發；且 toast 為浮層、比頂列的截斷灰字醒目數倍，實際被讀到的機率是提升而非降低。
- **[toast 疊在抽屜上方可能遮住抽屜內容]** → 單槽、6 秒自動消失、可提前關閉；且失敗當下 toast 的優先級本就高於抽屜內容。
- **[回歸對照]** → 本刀不觸及 CLI 人眼輸出、`--json` 欄位或 core API，parity_suite / color_suite / twin harness 三組對照皆不受影響，無需重新基線。
- **[跨平台]** → sonner 為純 React 套件、無平台相依 API；主題走 OS 的 `prefers-color-scheme`，Windows / macOS / Linux 行為一致。真實視窗驗證於任一平台執行即可涵蓋堆疊語意。

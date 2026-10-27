---
topic: 桌面 app 的操作回饋面：頂列狀態列的用途、生命週期，以及是否需要正式的 toast 概念
slug: desktop-feedback-surface
status: promoted
promoted_to: desktop-failure-toast
created: 2026-07-17
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 桌面 app 的操作回饋面：頂列狀態列的用途、生命週期，以及是否需要正式的 toast 概念

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者刪除一個變更後，視窗頂列出現「clarify-discussion-change-routing · 已刪除」並且不消失，質疑此面的用途——「感覺不是很重要的 alert 但又不完全是 alert」，並追問是否該正式引入 toast／alert 概念。

**模式**：assumptions——codebase scout 命中 4 個相關來源（`apps/desktop/src/store.ts`、`apps/desktop/src/App.tsx`、`openspec/specs/desktop-app/spec.md`、封存的 `2026-07-09-desktop-verb-drawer-surface` design／proposal），足以形成立場。

**相關規格**：`desktop-app/spec.md:156`——「視窗頂列狀態列 SHALL 保留供看板全域操作（刪除、封存、拖排失敗）之結果訊息」。

**相關變更（已封存）**：`2026-07-09-desktop-verb-drawer-surface`（把 validate／analyze 從頂列搬進抽屜，理由：「離抽屜按鈕很遠、使用者實測感覺沒反應」；design 明寫「不移除視窗頂列狀態列——其保留供看板全域操作」）、`2026-07-11-desktop-ux-polish`（重申同一句規格）。

**深度檢查**：不觸發——本題不引入新模組／新 IPC／新儲存抽象，只涉既有 UI 回饋面。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-17)

**Focus**: 頂列 `verbResult` 目前由什麼觸發、實際用途為何，以及為何讀起來「像 alert 又不是 alert」

**Position**: 它是動詞結果面搬家後留下的殘留面，無生命週期，且是全 app 唯一違反「回饋貼在觸發點旁」哲學的地方。
- 觸發點共 9 條（`store.ts`）：刪除變更成功/失敗（:428/:430）、封存討論成功/失敗（:451/:453）、封存變更成功/失敗（:470/:473）、拖排寫回失敗（:510）、開啟專案失敗（:620）、初始化專案失敗（:674）。渲染於 `App.tsx:313-314`（mono 灰字、`truncate max-w-[40%]`）。
- **無任何清除路徑**：`verbResult` 全檔只在初始 state（`store.ts:246`）設為 `null`，之後沒有 timeout、沒有關閉鈕，切分頁／切頁／關抽屜皆不清——只被下一次全域操作覆蓋，否則掛到關閉 app。這就是「像 alert 又不是」的成因：它有 alert 的語氣，卻有狀態列的持久性。
- 同一份 store 內對照組有生命週期：`drawerVerb` 有 `clearDrawerVerb()`（:499）且切 change 即失效。
- 來歷：頂列原是**所有動詞結果的唯一出口**（`formatVerbResult` 壓成一行）。`2026-07-09-desktop-verb-drawer-surface` 把 validate／analyze 搬走，判定的失敗理由正是「離抽屜按鈕很遠、使用者實測感覺沒反應」；但該刀範圍只到 validate／analyze，頂列連同其未經設計的短暫性語意被原地保留。
- 附帶違例：`archive ✓ <datedName>`（:470）混用英文動詞、未走 i18n，抵觸 `.claude/CLAUDE.md`「UI 文字硬編繁中」與 `LANGUAGE.md` 工程詞禁令（slug 例外只涵蓋討論識別錨點）；`openProjectAt`／`confirmInit` 寫入裸錯誤字串、無主詞前綴，且「專案級錯誤」不在 spec:156 授權的「看板全域操作」名單內。

**Open**: 成功回饋（4 條）是否為雜訊（看板變化本身即回饋）？失敗回饋（5 條）該貼回觸發點還是留在頂列？是否需要正式引入 toast／alert 概念？

### Round 2 — assumptions (2026-07-17)

**Focus**: 頂列狀態列是否根本就是 toast 的功能，該不該正式引入 toast／alert 概念

**Position**: 頂列被抽屜蓋住是實錘 bug（唯一看得見的訊息恰是唯一不需要的那條），且此事實推翻了我上一輪「不要 toast」的立場——改推 toast 但只給失敗。
- **實錘**：`SheetOverlay`（`packages/ui/src/components/ui/sheet.tsx:19`）是 `fixed inset-0 z-50 bg-black/40` 全視窗覆蓋；頂列 `<header>`（`App.tsx:286`）無 z-index，整條躺在黑幕下，而訊息本就是 `text-xs text-muted-foreground`。抽屜動作列正是「封存」（`RichDetailDrawer.tsx:319`）與「刪除」（:328）的觸發點。
- **可見性完全反了**：刪除成功清 `detailChange`（抽屜關 → 看得到，但這條最不需要，卡片消失即回饋）；刪除失敗不清（抽屜開 → 被蓋，但這條最需要）；封存成功／失敗皆不清（`runVerb` 從不清 `detailChange` → 兩條都看不到）。`spec.md` 的「寫回失敗不留假象」在此是空的。
- **修正一**：選項 A（貼回錨點）有兩個真的無錨點的缺口——拖排失敗（卡片跳回原位不解釋原因，卡片非錯誤歸屬對象）、開啟專案失敗（`pickFolder` 是已關閉的 OS 原生對話框，分頁尚未建立，`tabErrors` 救不了）。
- **修正二**：我把 `2026-07-09-desktop-verb-drawer-surface` 的教訓講得比實際範圍大。原文理由是「analyze 四維度發現項被壓成一個計數丟棄」＋「離按鈕很遠、感覺沒反應」——那是**有結構的成功結果被壓成一行**。單行失敗訊息不同：失敗時使用者正在找「為什麼沒動」，浮出的訊息比角落灰字更易被找到。該次結論精確講是「有結構的結果要進抽屜」，非「所有回饋必須 co-located」。
- **新立場**：成功 4 條靜默、失敗 5 條走 toast（portal `z-[60]` 浮於 sheet overlay 之上）、頂列移除、`spec.md:156` 改寫。原語自建不引 sonner，單槽不堆疊（失敗訊息不會併發，佇列是臆測性設計）。一刀同解：被蓋住、永久佔位、兩個無錨點缺口、`archive ✓` 英文違例，且不動 `alert-dialog` 行為。

**Ruled out**: 選項 C（頂列加 timeout）——治永久佔位但留著「被抽屜蓋住」與「離觸發點遠」，且無法救兩個無錨點缺口。選項 A 全套（成功失敗都貼回錨點）——拖排失敗與開啟專案失敗無錨點可貼，會逼出第二個回饋面、反而不一致。

**Open**: 成功靜默是否可接受（支點；使用者尚未回答）？拖排成功是否也該 toast——`spec.md:156` 對拖排已明寫「拖排**失敗**」、成功靜默是既生效的刻意決定？

### Round 3 — assumptions (2026-07-17)

**Focus**: toast 原語該自建還是走 shadcn 的 sonner——本輪查證推翻上一輪對 sonner 的否決理由

**Position**: 改用 sonner。上一輪把 sonner 列為 Rejected alternative 的理由有一項是事實錯誤，經查證後該否決不成立。
- **我的事實錯誤**：上一輪主張「shadcn 的 sonner 路徑帶 `next-themes` 相依且與本專案主題機制衝突」——錯。`next-themes` 是 shadcn **wrapper** 為 Next.js 而用的，不是 sonner 的相依；`npm view sonner` 顯示 sonner 2.0.7 的 `dependencies` 為空、peer 只有 react／react-dom（含 19）。sonner 本身零執行期依賴。
- **主題天然吻合**：桌面 app 的深色模式是 `apps/desktop/src/index.css` 的 `@media (prefers-color-scheme: dark)`（OS 驅動，非 `.dark` class 切換），與 sonner 的 `theme="system"` 預設正好一致——拆掉 shadcn wrapper 的 `useTheme()` 後無需任何主題接線。
- **核心修復免費且久經沙場**：sonner 的 toaster z-index 為 `999999999`（解自 `dist/styles.css`），遠高於本專案全部 `z-50` 的遮罩層。本刀唯一真正要修的「訊息被抽屜遮罩蓋住」不必自己算層級。
- **一致性代價消失**：`packages/ui/src/components/ui/` 註解白紙黑字寫「shadcn/ui 設計系統原語」，其餘 7 個原語全數出自 shadcn。自建 toast 會是唯一例外；走 sonner 則無此破口。
- **連帶推翻 D1**：sonner 的 `toast.error()` 是 module-level singleton、可在 React 外呼叫——而 D1 當初「狀態留在 store」的唯一論證正是「寫入點在 store 非元件、得把 hook 反向注入」。sonner 直接解掉該問題，故 store 不再需要 `failure` 狀態與 `dismissFailure()`，改為直接呼叫。淨結果是程式碼更少。
- **連帶簡化 D2／D3**：層級無須自訂；逾時、關閉鈕、hover 暫停計時皆由 sonner 提供（hover 暫停原列 Non-Goals，現為內建、不另付成本）。

**Ruled out**: 純 React 自建 toast——輸在：與 `components/ui/` 的 shadcn 慣例不一致（會是唯一非 shadcn 原語）；z 軸堆疊、hover 暫停、無障礙須自理；而其唯一優勢「零依賴」在 sonner 同為零執行期依賴的事實下不成立。`@radix-ui/react-toast`——輸在 shadcn registry 已下架 toast（`@shadcn/toast` 回 not found），不再是 shadcn 路徑。

**Open**: store 直接 import sonner 是否可接受——store 目前零 UI import（雖已 import i18n 組使用者可見字串）？替代為 store 保留 `failure` 狀態＋App 以 effect 轉呼 sonner，但屬雙重記帳且相同訊息連續兩次不重觸發、須再加 id 計數器。

## Conclusion

**Decision**: 移除視窗頂列狀態列（`store.verbResult` 及 `App.tsx` 頂欄的渲染），改為「畫面說了沒有」判準——畫面已表達的結果靜默，畫面未表達的才 toast；toast 採 shadcn registry 的 sonner。

- **更正輪次的數字**：前兩輪記為「成功 4 條、失敗 5 條」有誤，實際為 **3 成功 / 6 失敗**。拖排成功本就不寫 `verbResult`（`store.ts` 僅失敗路徑寫入），不在這 9 條內——它是既生效的靜默行為，維持不變。
- **3 條成功寫入刪除、改為靜默**：刪除成功（卡片消失即回饋）、討論封存成功、封存變更成功（卡片消失＋側欄「已封存」計數遞增）。`archive ✓ <datedName>` 一併消失，順帶解掉其混用英文動詞、未走 i18n 的規範違例；`datedName` 於「已封存」頁可見，非唯一出口。
- **6 條失敗寫入改走 toast**：刪除失敗、討論封存失敗、封存變更失敗、拖排寫回失敗、開啟專案失敗、初始化專案失敗。
- **toast 採 sonner（Round 3 反轉，取代原「自建、不引 sonner」）**：走 shadcn registry 的 `@shadcn/sonner` 路徑，拆除其 wrapper 的 `next-themes` 相依（sonner 以 `theme="system"` 對接 app 既有的 `prefers-color-scheme` 深色模式）。sonner 的 z-index `999999999` 遠高於本專案全部 `z-50` 遮罩層，直接解掉「訊息被抽屜遮罩蓋住」；逾時、關閉鈕、hover 暫停皆內建。以固定 toast id 呼叫達成「單槽、新訊息取代前一則」語意。
- **store 不再保留失敗狀態**：sonner 的 `toast.error()` 為 module-level singleton、可在 React 外呼叫，故 `verbResult` 直接刪除、不以 `failure` 狀態取代，亦無 `dismissFailure()`。
- **封存成功必須清 `detailChange`（關抽屜）**：目前不清，若成功靜默而抽屜仍開著蓋住看板，「畫面說了」的前提即不成立、使用者會「感覺沒反應」。刪除成功已清，封存漏了——此條是判準成立的必要條件，非附帶優化。
- **`desktop-app` 規格中「視窗頂列狀態列 SHALL 保留供看板全域操作之結果訊息」須改寫**，改述失敗 toast 面與成功靜默契約。

**Rationale**: 決定性事實是頂列在最需要時看不見——抽屜遮罩為 `fixed inset-0 z-50` 全視窗覆蓋，頂欄無 z-index，而「封存」與「刪除」正是從抽屜觸發。可見性完全反了：刪除成功清 `detailChange`（抽屜關→看得到，卻是最不需要的一條），刪除／封存失敗不清（抽屜開→被蓋，卻是最需要的一條）。關鍵取捨在「成功要不要回饋」：判準取「畫面說了沒有」而非「有沒有操作」，因為 toast 的價值來自稀缺——每個操作都 toast 會讓使用者在數日內學會無視該角落，真正的失敗訊息就被埋進重複噪音，等於換一種方式回到「看不見的回饋面」。實作面採 sonner 而非自建：`components/ui/` 全數為 shadcn 原語，自建會是唯一例外；且 sonner 零執行期依賴，「不引依賴」的反對論證不成立。

**Rejected alternatives**:
- **全部 9 條都 toast（含拖排成功）**——使用者初始傾向，經討論後改採失敗才 toast。輸在：拖排的回饋是卡片跟著手到新位置（介面能給的最直接回饋），文字加不了資訊；且會反轉 `board-card-order` 對拖排明寫「拖排**失敗**」的既有刻意決定；重複噪音會侵蝕 toast 面的可信度。
- **成功失敗都貼回觸發點、不引 toast**——輸在兩個無錨點缺口：拖排失敗（卡片跳回原位不解釋原因，卡片非錯誤歸屬對象）、開啟專案失敗（原生選檔對話框已關閉、分頁尚未建立）。硬推會逼出第二個回饋面、反而不一致。
- **頂列加逾時自動消失**——輸在只治永久佔位，留著「被抽屜蓋住」與「離觸發點遠」，且無法救兩個無錨點缺口。
- **純 React 自建 toast**（Round 2 曾採納、Round 3 推翻）——輸在與 `components/ui/` 的 shadcn 慣例不一致（會是唯一非 shadcn 原語），且 z 軸堆疊、hover 暫停、無障礙須自理；其唯一優勢「零依賴」在 sonner 同為零執行期依賴的事實下不成立。**Round 2 對 sonner 的否決含一項事實錯誤**（誤稱 sonner 帶 `next-themes` 相依且與本專案主題機制衝突——`next-themes` 實為 shadcn wrapper 所需，非 sonner 相依），該否決不成立。
- **`@radix-ui/react-toast`**——輸在 shadcn registry 已下架 toast（`@shadcn/toast` 回 not found），不再是 shadcn 路徑。

**Deferred**: none——三輪的 open question 皆已裁定：成功靜默可接受（含拖排成功維持靜默）；store 直接呼叫 sonner 的 imperative API，不保留失敗狀態（替代的「store 保留 `failure` ＋ App 以 effect 轉呼」屬雙重記帳、且相同訊息連續兩次不重觸發須再加 id 計數器）。

**Capture to**: proposal（變更 `desktop-failure-toast`，已轉出）

**Next**: 變更 `desktop-failure-toast` 的 artifacts 依本結論更新後 /speclink-apply

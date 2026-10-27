## Context

desktop-async-commands 已把觸及檔案系統／子進程的 Tauri command 全數 async 化，整窗不再凍結，但載入期間的視覺回饋從未實作：

- store 的 WorkspaceSnapshot.loaded 旗標（apps/desktop/src/store.ts）有寫入、全 repo 零 UI 消費者。
- 切換 workspace 分兩段等待：A 段——activateTab 對本地分頁先 await openProject probe（spawn 子進程，macOS 首抓可秒級），成功才翻 activeKey，期間主視窗分頁列與 tray 面板分頁高亮完全不動；B 段——activeKey 翻轉後顯示該 workspace 舊快取、首訪為空清單，refresh 四路並發完成才有真資料。首訪空清單與「真的沒東西」無法區分。
- 詳情抽屜的文件狀態已有三態慣例（undefined＝載入中、null＝檔不存在、字串＝內容），但消費不一致：變更抽屜 proposal 分頁、規格抽屜、已封存抽屜討論頁顯示「載入中」文字；變更抽屜 design、tasks 等分頁把載入中直接渲染成「沒有文件」假空態。
- tray 面板是 TraySnapshot 的薄渲染層（與原生選單同源自主視窗 store），無獨立資料路徑。

來源討論：desktop-loading-skeleton-ux（結論與淘汰項見該記錄）。

## Goals / Non-Goals

**Goals:**

- A 段：分頁切換中在目標分頁給即時 spinner 回饋（主視窗分頁列＋tray 面板分頁條）。
- B 段：清單首訪（loaded 為 false）時看板欄與 tray 面板分區以 skeleton 佔位卡呈現。
- 文件層：四個抽屜統一消費 undefined 載入態並渲染文件 skeleton，消除假空態。
- skeleton 基元與文件 skeleton 組件落 packages/ui，desktop 與 server-web 共用。

**Non-Goals:**

- 樂觀翻頁（probe 失敗回滾與 tabErrors 錯誤歸屬複雜化）。
- refresh 期間的 skeleton 或「更新中」指示——有舊快取一律顯示舊資料、靜默更新。
- 原生 tray 下拉選單的載入呈現（平台無此能力）與 tray 圖示忙碌變體。
- 讀取提速（worktree 觀察面每次現取維持不變）；watcher 事件節流。
- server-web 端消費 skeleton（基元共用即可，接線另案）。
- remote 分頁的連線中呈現——既有 recovery 狀態機（正在連線／離線等）已覆蓋，不疊加 spinner。

## Decisions

**D1. A 段回饋＝store 新增 pendingTabKey，不改切換順序**
activateTab 本地路徑進 probe 前設 pendingTabKey 為目標分頁 key；**probe settle（成功或失敗）當下即清為 null**——spinner 只涵蓋探測本身，翻頁後的整批載入由 B 段的 skeleton 承擔；兩者並存會讓已切換完成的分頁持續掛「正在切換」。主視窗 ProjectTabs 與 tray 面板分頁條對 key 相符的分頁渲染小 spinner。remote 分頁不設：既有 session 直翻無 probe 段、重連走 recovery 狀態機。
替代方案：樂觀翻頁——被討論淘汰（回滾複雜）；只改游標樣式——tray 面板與主視窗分屬不同視窗，游標回饋不可靠也不指向目標分頁；spinner 撐到整批載入結束——與 skeleton 語意重疊，且 aria-label 會在切換完成後仍念「正在切換」。

**D2. B 段 skeleton 條件＝「載入正在進行」且「尚無真值」**
條件為 `(探測中 || 刷新中) && !loaded`：loaded 只說「有沒有真值」，讀取失敗時它必須維持 false（讀不到 ≠ 確認是空的），故骨架的**終止**由進行中旗標負責；loaded 為 true（含舊快取）照常渲染資料，refresh 完成靜默換新。store 新增 `refreshing`（整批載入進行中），TraySnapshot 對應 `workspaceRefreshing`；旗標隨活躍 workspace 重置。翻頁入口逐一表態「後面接不接整批載入」（willRefresh 必填參數）：會接的入口在 activeKey 翻轉當下即標記載入中（避免「已翻頁、尚未開跑」的空窗渲染成真空態）；不接的入口（開修復頁、重連 handshake——連線中由既有 recovery 狀態機遮罩）一律不標——標了沒人收，骨架就永久掛著。
單以 loaded 為條件——被淘汰：探測失敗或讀取失敗的 workspace 會永遠停在骨架，把「有東西正在載」退化成「永遠在載」。單以 refresh 進行中為條件——被淘汰：watcher 與切換共用 refresh，每次刷新都閃爍；合取 `!loaded` 後首訪即定案，watcher 刷新不再觸發骨架，此淘汰理由已被涵蓋。世代計數判斷「切換後首輪」——過度設計。

**D3. 文件 skeleton 在抽屜層條件渲染，不動 SectionedDoc 的 empty 語意**
四個抽屜（變更 RichDetailDrawer、規格 SpecDrawer、討論 DiscussionDrawer、已封存 ArchivedDrawer）在文件狀態為 undefined 時渲染 DocSkeleton 組件，undefined 以外照現行路徑（null 走空態文案、字串走內容渲染）。SectionedDoc／Markdown 的 empty 參數語意不變——「載入中」文字自 empty 參數移除，該文字的假借用法（把載入中當空態）一併消失。
替代方案：把載入態下沉到 SectionedDoc——被淘汰（SectionedDoc 是純內容渲染器，感知載入態會讓資料源語意滲入渲染層，違反前端元件庫與資料源解耦要求）。

**D4. skeleton 基元落 packages/ui 的 shadcn 基元群**
新增 packages/ui/src/components/ui/skeleton.tsx（shadcn Skeleton：圓角灰塊＋pulse 動畫，尊重 prefers-reduced-motion 時停用動畫）；三種佔位組件——看板佔位卡（卡形）、面板佔位列（列形）、DocSkeleton（標題條＋數行內文條）——以基元組合，集中於 packages/ui/src/components/skeletons.tsx 並自 packages/ui/src/index.ts 匯出，供兩端共用。佔位卡的**外框沿用 Card 基元、不自刻描邊**：佔位卡是「還沒到的那張真卡片」，手刻表面樣式遲早與真實卡片分歧——本專案為 Tailwind v4，裸寫 `border` 的顏色落到 currentColor（近黑描邊），2026-08-12 手動驗收抓到的正是這個。
替代方案：放 apps/desktop——被淘汰（server-web 共用不到）。

**D5. tray 面板狀態經 TraySnapshot 同源**
TraySnapshot 新增 pendingTabKey（string | null）、workspaceLoaded 與 workspaceRefreshing（boolean，活躍快照的 loaded 與 store 的 refreshing）三欄，由 buildTraySnapshot 自 store state 導出；面板據以渲染分頁 spinner 與分區 skeleton。面板不自建任何載入狀態，維持薄渲染原則。既有的 remote recovery 遮資料優先於 skeleton——面板側對應物是 TrayPanel 的 activeRecovery 分支（原生選單側為 hideWorkspaceData），遮資料時不出骨架。切換中與活躍分頁、載入態走即時推送（不進去抖），清單內容仍走去抖——spinner 消失與高亮移轉須同批抵達，否則面板會閃一下 spinner 就回到切換前的樣子。例外：workspaceRefreshing 不進即推面——成功路徑由 loaded 翻真同批抵達；首訪讀取失敗時面板骨架的收掉等一個去抖週期，換取清單內容維持去抖不被載入旗標帶穿。

**D6. spinner 與 skeleton 皆附無障礙標記**
spinner 帶 aria-label（新 i18n 鍵，語意「切換中」）；skeleton 區塊帶 aria-busy，載入完成後消失。既有「載入中」i18n 文字的消費點汰換後，若該鍵無其他消費者則一併移除。

## Implementation Contract

**Behavior（完成後可觀察行為）**

- 點擊主視窗分頁列或 tray 面板分頁條上的非活躍本地分頁：目標分頁立即（同一事件迴圈批次內）出現 spinner；probe settle 當下 spinner 消失——成功則同批翻頁並接手為 skeleton，失敗則分頁錯誤呈現照舊。
- 切入首訪 workspace：看板四欄（提案中／進行中／已就緒／討論）欄名照常、卡片區為 skeleton 佔位卡；tray 面板分區標題照常、內容列為 skeleton 列；refresh 完成後換為真資料（或真空態文案）。
- 切入已訪 workspace：直接顯示舊快取資料，全程無 skeleton；refresh 完成靜默更新。
- 開啟任一抽屜的任一文件分頁：內容抵達前顯示文件 skeleton；載入完成後——有內容顯示內容、檔案不存在顯示該分頁既有空態文案。任何分頁不再於載入中顯示「沒有文件」類文案。
- 動畫在 prefers-reduced-motion 下停用（靜態灰塊）。

**State／Interface**

- store（apps/desktop/src/store.ts）：AppState 新增 pendingTabKey: string | null（初值 null；僅 activateTab 本地路徑寫入）與 refreshing: boolean（初值 false；整批載入進行中，隨活躍 workspace 重置）。翻頁入口以必填參數表態接不接整批載入：會接且尚無真值 → activeKey 翻轉當下即為 true；不接（開修復頁、重連 handshake）→ 一律 false。
- TraySnapshot（apps/desktop/src/tray.ts）：新增 pendingTabKey、workspaceLoaded 與 workspaceRefreshing 欄位；panel 經既有 tray-snapshot 事件接收，無新事件、無新 IPC command。
- packages/ui 新增匯出：Skeleton 基元、看板佔位卡、面板佔位列、DocSkeleton。既有元件 props 僅允許新增可選項，不做破壞性變更。

**Verification**

- 單元測試（vitest）：store 的 pendingTabKey 生命週期（設定→探測完成清除；設定→失敗清除）與 refreshing 的記帳邊界（關閉在途分頁、跨 workspace 不互清、翻頁與標記同批、不接載入的入口不標）；buildTraySnapshot 三個新欄位導出；KanbanBoard 於 loading prop 為 true 渲染 skeleton、false 渲染資料（App 端以「(探測中 || 刷新中) && !loaded」導出）；四抽屜各文件分頁三態（undefined→skeleton、null→空態文案、字串→內容）；TrayPanel 分頁 spinner 與分區 skeleton。
- 既有測試全綠：apps/desktop 與 packages/ui 的 vitest 套件。
- 手動驗收（[M] 任務）：tray 切換至忙碌 repo 的 workspace，確認 spinner 即時出現、期間 UI 可互動、首訪出 skeleton、已訪不閃 skeleton。

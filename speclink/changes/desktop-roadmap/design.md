## Context

討論 herdr-speclink-panel 定案「Herdr 管執行、speclink 本體管規劃與閱讀」，本變更是第 3 刀。它依賴的第 1 刀 desktop-deep-link（`speclink://` 網址）與第 2 刀 plan-unpromoted-cuts（`plan` 的 `unpromotedCuts`）都已封存。現況：

- 桌面沒有 plan 查詢。local 的 Tauri command `list_changes` 委派 speclink-desktop-core 的 `query::list_changes_at`，它以 `board_order(store)` 取得引擎 `plan::compute` 的結果，只把每列的六個排程欄位（`PLAN_FIELDS`：wave、blockedBy、dependsOn、overlaps、requirementOverlap、archiveAfter）疊到清單項上，回應頂層只有 `planError`；`Plan` 的 `waves`、`next`、`skipped`、`unpromotedCuts` 都被丟掉
- remote 的 `RemoteWorkspace::list_changes` 取 GET /plan 後以 `merge_plan` 組裝 `RemoteChangeList { changes, plan_error }`；protocol 的 `PlanResponse` 已帶 `next` 與 `unpromoted_cuts`（舊 server 缺鍵讀作空陣列），同樣被丟掉
- 前端 `ChangeListPayload { changes, planError }`（packages/ui/src/adapter.ts）；store 的 `changes` 與 `planError` 由 `refresh()` 填入並依 workspace 快取（`WorkspaceSnapshot`）
- 看板（`KanbanBoard`）依階段分欄，卡片以波次章、變淡與 tooltip 表示排程；packages/ui/src/stage.ts 已有 `changeStage`、`planWave`、`planBlockedBy`、`planWaveLabel`、`planBlockedLabel` 與階段色表
- 主區頁面由 `BoardView`（board、specs、manual、archived、project-settings、settings）決定；`openDetail`、`openDiscussion` 在同一次狀態更新把 `boardView` 設回 "board"（desktop-app「變更與討論詳情頁的宿主為看板」），詳情頁的 ‹ › 序列來自 `KanbanBoard` 的 `onVisibleOrderChange`
- 「尚未轉出」的橄欖色虛線樣式在討論詳情頁的刀卡（`CutCardView`：`border-dashed` 加 `SEMANTIC_SURFACE.manual`、`StatusPill tone="manual"`）；轉出指令以 `CommandChip` 呈現並可複製
- 網址解析在 apps/desktop/src/deepLink.ts：`parseSpeclinkUrl` 只接受 host `change`、`discussion`，`openLink` 依序喚起主視窗、檢查不可取消的浮層、`openProjectAt`、開詳情
- packages/ui 沒有任何畫線或圖形元件，也沒有圖形函式庫

## Goals / Non-Goals

**Goals:**

- 看板頁加路線圖檢視：每一波一欄、每個變更一站、依賴線、`next` 標示、滑鼠停留或焦點時整條路線亮起
- 同一張圖列出尚未轉出的刀，並可複製轉出指令
- `speclink://roadmap?project=<path>` 開啟該專案的路線圖
- local 與 remote 專案行為相同

**Non-Goals:**

- Herdr 內的路線圖、刀與刀之間的依賴、agent 狀態
- 路線圖上的寫入操作（改依賴、拖排、轉出）
- 新的鍵盤快捷鍵與應用程式選單項
- server-web；CLI 指令與輸出

## Decisions

### 清單回應頂層增列 next 與 unpromotedCuts，不另開 plan 查詢

local 的 `query::list_changes_at` 在 plan 計算成功時，回應頂層增列 `next`（plan 的 `next`，字串或 null）與 `unpromotedCuts`（plan 的 `unpromoted_cuts` 原樣序列化，每項 `discussion`、`cut`、`title`、`changes`、`unpromoted`）；成環時 `next` 為 null、`unpromotedCuts` 為空陣列。remote 的 `RemoteChangeList` 增列 `next: Option<String>` 與 `unpromoted_cuts: Vec<PlanUnpromotedCut>`（camelCase 序列化為同兩鍵），`merge_plan` 自 `PlanResponse` 逐欄搬運；plan 端點回 409 或其他錯誤時兩鍵為 null 與空陣列。值都取自引擎 plan 的同一次計算，前端不另算。

理由：路線圖要的波次、阻擋、依賴、階段、任務進度都已在清單項上，只缺 `next` 與尚未轉出的刀；放進同一個回應，路線圖與看板吃同一次載入與檔案監看，不會一邊新一邊舊。替代方案：新增 `plan` Tauri command——多一次查詢與快取欄位，兩份資料可能不同步。

### 路線圖是看板頁的一種檢視

store 新增 `boardMode: "cards" | "roadmap"`（app 層級、預設 "cards"、不寫入磁碟）與 `setBoardMode`；`BoardView` 不變。`boardView` 為 "board" 時，主區同時掛載 `KanbanBoard` 與 `RoadmapView`，只顯示 `boardMode` 指定的那一個（另一個保持掛載但不可見、不可互動）。看板頁的頁首放「看板｜路線圖」切換，沿用 @speclink/ui 的 `Tabs` segmented 變體，包成 `BoardViewSwitch`，`KanbanBoard` 新增 `viewSwitch` prop 放在搜尋列之前，`RoadmapView` 的頁首放同一個元件。

結果：專案欄的「變更」項照舊高亮、側欄五項不變；`openDetail`／`openDiscussion` 照舊只把 `boardView` 設回 "board"、不動 `boardMode`，所以自路線圖開詳情頁、關閉後回到路線圖；`KanbanBoard` 保持掛載，看板的捲動、搜尋、篩選與 ‹ › 序列在路線圖檢視下照舊（‹ › 依看板順序，不另定一套）。替代方案：`BoardView` 加 "roadmap"——要在專案欄加導覽項（改「側欄導覽結構」與選單快捷鍵編號），且開詳情會把宿主切回卡片看板，關閉後回不到路線圖。

### 路線圖模型為純函式

新增 packages/ui/src/roadmapModel.ts：

- `roadmapModel(changes, next)` → `{ waves: RoadmapWave[]; edges: RoadmapEdge[]; unplaced: string[] }`。`RoadmapWave { wave: number; stations: RoadmapStation[] }` 依波次升冪；`RoadmapStation { name; stage; completedTasks; totalTasks; blockedBy: string[]; next: boolean }`，同一波內依清單順序（即 plan 的配置序），`stage` 用 `changeStage`、`blockedBy` 用 `planBlockedBy`；`RoadmapEdge { from: 前置; to: 依賴者 }` 只收兩端都在路線圖上的 `dependsOn`；`unplaced` 是沒有 `wave` 的變更（壞 metadata）
- `roadmapRoute(model, name)` → `{ stations: Set<string>; edges: Set<string> }`：該站本身、往前的全部遞移前置、往後的全部遞移依賴者，以及兩端都在這組站內的線（線的鍵為 `from→to`）

理由：分組、連線與路線的判定可在 vitest 以資料驗證，不必依賴版面量測；元件只負責畫。

### 依賴線以 SVG 疊層繪製

`RoadmapView` 以 ref 記下每站的 DOM 節點，在資料變動與捲動內容尺寸改變（`ResizeObserver`）時量測位置，於捲動內容上疊一層絕對定位、不接收指標事件的 `<svg>`，每條線畫成三次貝茲曲線：起點為前置站的右緣中點，終點為依賴者站的左緣中點。依賴必定指向較後的波次，所以線一律由左往右。

替代方案：引入圖形函式庫——只有一張圖、幾何簡單，不值得多一個依賴；以固定站高推算座標——站名換行時位置會錯。

### 站的外觀表示階段，next 標示下一個

每站是一顆按鈕（可 Tab 聚焦、Enter 開詳情）：首行變更名（等寬字）；次行依階段：已就緒用 `STAGE_BADGE.ready` 的實心樣式並寫「已就緒」；進行中以小圓圈表示任務進度（`completedTasks / totalTasks`，以 SVG 圓弧繪製）並寫 n/m；提案中為外框樣式並寫 n/m；`blockedBy` 非空的站整站變淡並寫「等 a、b」（`planBlockedLabel`）。站名等於 `next` 時加「下一個」籤與主色外框。

欄頭寫「第 N 波」（`planWaveLabel`）與變更數，第 1 波另寫「可以並行」，第 N 波（N ≥ 2）寫「等第 N−1 波」。`unplaced` 非空時在頁首下方以一行小字說明「N 個變更無法排程，只在看板顯示」。

### 路線高亮

滑鼠移入或焦點落在一站時，以 `roadmapRoute` 取得路線：路線上的站與線維持原樣（線改主色），其餘站與線變淡；移出或失焦時恢復。只在畫面上改透明度，不改資料。

### 尚未轉出的刀放在最右一欄

`unpromotedCuts` 非空時，最右多一欄「尚未轉出」（寫刀數），每把刀一張卡：卡頭為所屬討論 slug 與刀編號（例：`herdr-speclink-panel · cut 4`）加 `StatusPill tone="manual"` 的「尚未轉出」；其下為 `unpromoted` 的變更名（等寬字）與範圍句 `title`；卡底為 `CommandChip`，指令為 `/speclink-propose --from-discussion <slug>`。外框沿用 `border-dashed` 加 `SEMANTIC_SURFACE.manual`。討論 slug 在目前專案的討論清單內時可點，點擊以 `openDiscussion` 開該討論的詳情頁；不在清單內時為純文字。不畫刀與刀、刀與變更之間的線。`unpromotedCuts` 為空時沒有這一欄。

### 成環與空狀態

`planError` 非空時，路線圖不畫波次欄與線，改顯示與看板相同的成環提示（沿用 i18n `board.planCycle` 與同款警示框），尚未轉出的刀欄照舊依 `unpromotedCuts`（成環時為空）。沒有任何已排程的變更且沒有尚未轉出的刀時，顯示空狀態「沒有進行中的變更」。資料載入中沿用看板的載入狀態。

### 網址加 roadmap 這種

`parseSpeclinkUrl` 加 host `roadmap`：path SHALL 為空（`speclink://roadmap?project=…` 或 `speclink://roadmap/?project=…`），有任何名稱段即為格式錯誤（reason `name`）；`project` 規則同既有。解析結果型別改為聯集：`{ kind: "change" | "discussion"; name; project }` 或 `{ kind: "roadmap"; project }`。`openLink` 的前段（喚起主視窗、格式錯誤 toast、不可取消的浮層、`openProjectAt`）不變；`openProjectAt` 回傳非 null 後，roadmap 的處理為關閉開著的詳情頁、`setBoardView("board")`、`setBoardMode("roadmap")`，沒有找不到目標的情形、不出 toast。Rust 端的外掛註冊與網址格式設定不變。

### 文字走 @speclink/ui 的 i18n

路線圖、切換鈕、尚未轉出欄與空狀態的文字加進 packages/ui/src/i18n.tsx（tw、en），元件以 `useI18n` 取用；沿用既有鍵 `card.wave`、`card.blockedTitle`、`board.planCycle`、`conclusion.notPromoted`、`conclusion.copyCommand`。網址這次沒有新的 toast，apps/desktop 的 messages.ts 不動。

沿用的既有元件與工具：`PageHeader`、`Tabs`（segmented）、`StatusPill`、`CommandChip`、`SEMANTIC_SURFACE`／`SEMANTIC_TONE`、`STAGE_BADGE`／`STAGE_BAR`、`changeStage`、`planWave`、`planBlockedBy`、`planWaveLabel`、`planBlockedLabel`。新增的 `RoadmapView`、`BoardViewSwitch` 放在 @speclink/ui，apps/desktop 只組裝。

## Implementation Contract

**行為**

- 看板頁頁首有「看板｜路線圖」切換；選路線圖後主區為路線圖，專案欄「變更」項仍高亮；再選看板回到卡片，捲動、搜尋與篩選與切換前相同
- 路線圖每一波一欄、每個變更一站，依賴線由前置連到依賴者；`next` 的站有「下一個」籤；被擋的站變淡並寫出在等誰
- 滑鼠停留或焦點在一站時，該站的遞移前置、遞移依賴者與其間的線維持原樣，其餘變淡
- 點一站開該變更詳情頁；Esc 或返回後回到路線圖；詳情頁的 ‹ › 依看板順序
- 有尚未轉出的刀時最右一欄列出，卡上可複製 `/speclink-propose --from-discussion <slug>`；點討論 slug 開該討論詳情頁
- 依賴成環時顯示成環提示、不畫波次
- `speclink://roadmap?project=<path>` 讓主視窗出現並取得焦點、活躍分頁為該專案、主區為路線圖、沒有開著的詳情頁
- remote 專案的路線圖資料來自 GET /plan，行為與 local 相同

**介面**

- local 與 remote 的變更清單回應頂層：`changes`、`planError`（既有），新增 `next`（字串或 null）與 `unpromotedCuts`（陣列，每項 `discussion`、`cut`、`title` 字串與 `changes`、`unpromoted` 字串陣列）
- TS：`ChangeListPayload` 增 `next?: string | null`、`unpromotedCuts?: UnpromotedCutItem[]`（缺席讀作 null 與空陣列）；store 增 `planNext`、`unpromotedCuts`（隨 `WorkspaceSnapshot` 快取）、`boardMode`、`setBoardMode`
- `roadmapModel(changes, next)`、`roadmapRoute(model, name)`（見「路線圖模型為純函式」）
- `RoadmapView` props：`changes`、`next`、`unpromotedCuts`、`planError`、`discussions`、`onOpenChange(name)`、`onOpenDiscussion(slug)`、`viewSwitch`、`loading`；`BoardViewSwitch` props：`value`、`onChange`
- `parseSpeclinkUrl` 的成功結果為 `{ kind: "change" | "discussion"; name; project }` 或 `{ kind: "roadmap"; project }`；失敗 reason 不變

**失敗模式**

- plan 成環：`next` 為 null、`unpromotedCuts` 為空，路線圖顯示成環提示
- remote server 沒有 plan 端點（404）或連線失敗：`next` 為 null、`unpromotedCuts` 為空、清單項沒有波次，路線圖顯示空狀態
- 舊 server 的 plan 回應沒有 `unpromotedCuts`：讀作空陣列（protocol 既有預設）
- 沒有 `wave` 的變更：不畫站，頁首下方一行說明
- 討論 slug 不在目前專案的討論清單：純文字，不可點
- roadmap 網址帶名稱段：格式錯誤 toast，不切換專案；不可取消的浮層開著：只喚起主視窗；網址處理零寫入

**驗收**

- cargo test -p speclink-desktop-core：`list_changes_at` 的回應頂層帶 `next` 與 `unpromotedCuts`（有一份討論結論含未轉出的刀時列出該刀）、成環時為 null 與空陣列
- cargo test -p speclink-desktop（remote.rs 的 `merge_plan` 測試）：plan 帶 `next` 與 `unpromoted_cuts` 時搬到 `RemoteChangeList`；409 與 404 時為 None 與空
- vitest（packages/ui）：`roadmapModel` 的分組、波內順序、連線只收兩端都在、`unplaced`；`roadmapRoute` 的遞移前置與依賴者；`RoadmapView` 的欄頭、站的階段文字、「下一個」籤、被擋變淡、尚未轉出卡的指令、hover 時路線外的站帶變淡標記、點站呼叫 `onOpenChange`、成環提示、空狀態
- vitest（apps/desktop）：store 的 `boardMode` 在 `openDetail` 與關閉詳情頁後不變、快照帶新欄位；`parseSpeclinkUrl` 的 roadmap 合法與不合法網址；`installDeepLinks` 處理 roadmap 網址時呼叫 `openProjectAt` 後切到路線圖、不出 toast；remoteDataSource 讀到新兩鍵
- 手動：dev 視窗開本 repo，路線圖的欄、站、線與 hover 亮起正確；終端機執行 open 指令送出 roadmap 網址後主區為路線圖

**範圍**

- 範圍內：speclink-desktop-core 的清單回應、Tauri 殼 remote 的清單組裝、@speclink/ui 的路線圖元件與模型與 i18n、apps/desktop 的 store、App 組裝、deepLink.ts
- 範圍外：speclink-core、speclink-cli、server、server-web、Herdr mod；快捷鍵與選單；路線圖上的寫入操作

## Risks / Trade-offs

- [jsdom 沒有版面，測不到線的座標] → 分組、連線、路線放在純函式並以 vitest 驗證；元件測試只驗 `<path>` 的數量與路線外的變淡標記；座標由手動在 dev 視窗驗收
- [變更很多時欄數多、線交錯] → 欄寬固定、主區水平捲動；線在捲動內容座標系內，捲動不需重算，只在尺寸改變時重算
- [路線圖與看板同時掛載] → 兩者吃同一份 store 資料、不各自查詢；隱藏的一方不可互動
- [回歸對照] → 不動 CLI、core 與 golden；`list --json` 與 `plan --json` 不變；desktop-core 既有清單測試只多頂層兩鍵，既有鍵與各項欄位不變
- [跨平台] → 只動前端與 serde 序列化，Windows／macOS／Linux 行為相同；網址的系統登錄沿用 desktop-deep-link，不改

## Migration Plan

- 無資料遷移；舊版 app 不認得 roadmap 網址時照既有規則以 toast 說明網址無法開啟
- 退版：移除切換與 `RoadmapView` 即可，清單回應多出的兩鍵沒有其他讀者

## Open Questions

(none)

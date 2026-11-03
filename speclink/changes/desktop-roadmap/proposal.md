## Why

一次推進好幾個變更時，開發者最常要回答的規劃問題是「先做哪個、誰在等誰、下一刀轉出了沒」。這些資料引擎都已算好（`plan` 的波次、阻擋、`next`，以及 plan-unpromoted-cuts 新增的尚未轉出的刀），但桌面 app 只把波次和阻擋拆成每張卡上的小章，尚未轉出的刀則要逐一打開討論詳情頁才看得到。討論 herdr-speclink-panel 定案「Herdr 管執行、speclink 本體管規劃與閱讀」，本變更是其中的第 3 刀：在桌面看板這一層加一張路線圖，一張圖看完全部進行中的變更與尚未轉出的刀，並讓 Herdr 等外部程式用 `speclink://` 網址直接打開它。

## What Changes

- 看板頁加一個「看板｜路線圖」切換：路線圖是看板頁的另一種檢視，專案欄的「變更」項照舊高亮，側欄五個導覽項不變
- 路線圖每一波一欄（「第 N 波」、變更數、第 1 波標「可以並行」、其後標「等第 N−1 波」），每個變更一站；站的樣子表示階段：已就緒、進行中（以圈表示任務進度並寫 n/m）、提案中、等前一波（變淡並寫出在等誰）；plan 的 `next` 標「下一個」
- 依賴畫成線（前置的站連到依賴它的站）；滑鼠停留或鍵盤焦點落在一站時，該站往前與往後的整條路線亮起，其他站與線變淡
- 最右一欄列出尚未轉出的刀：沿用討論詳情頁「尚未轉出」的橄欖色虛線樣式，寫出所屬討論與刀編號、尚未轉出的變更名與範圍句，附「複製轉出指令」鈕（`/speclink-propose --from-discussion <slug>`）；點討論名稱開該討論的詳情頁
- 點一站開該變更的詳情頁；關閉詳情頁回到路線圖
- 依賴成環時路線圖不畫波次，改顯示與看板相同的成環提示
- 桌面的變更清單回應（local 與 remote）在頂層增列 plan 的 `next` 與 `unpromotedCuts`，路線圖與看板吃同一次載入，不另開查詢
- `speclink://` 網址加一種：`speclink://roadmap?project=<專案資料夾絕對路徑>` 開啟該專案的路線圖
- 相容性影響：CLI 指令與輸出不變（人眼與 `--json` 皆不變、不影響回歸對照）；桌面變更清單回應只在頂層新增兩鍵，既有鍵與各項欄位不變；舊 server 沒有 `unpromotedCuts` 時讀作空陣列

## Non-Goals

- 不在 Herdr 內做路線圖：討論已否決，Herdr 以網址跳到桌面
- 不畫刀與刀之間的依賴：結論的刀序不一定代表依賴，尚未轉出的刀只標所屬討論與刀編號
- 不加鍵盤快捷鍵與應用程式選單項：入口為看板頁的切換與網址；需要時另提變更
- 不畫 agent 狀態：桌面 app 不知道 Herdr 的 agent
- 不在路線圖上做寫入操作（改依賴、拖排、轉出）：路線圖只看；改依賴照舊在詳情頁的排程分頁
- 不改 server-web：server 後台沒有看板

## Capabilities

### New Capabilities

- `desktop-roadmap`: 桌面看板頁的路線圖檢視——波次欄、變更站、依賴線、`next` 標示、路線高亮，以及尚未轉出的刀。掃描到的鄰近規格：`desktop-app` 規定看板欄位、卡片的波次與阻擋章、詳情頁宿主與網址入口，`change-plan` 規定 plan 的資料，`desktop-manual-page` 是「看板層級新頁另立 capability」的前例；沒有一份規格描述以波次為欄、畫依賴線的總覽，desktop-app 已逾四千行，路線圖的版面規則另立一份較易維護

### Modified Capabilities

- `desktop-app`: 「以 speclink 網址開啟詳情頁」加 `roadmap` 這種網址
- `client-protocol`: 「變更清單的排程欄位」與「remote 變更清單的排程欄位」的回應頂層增列 `next` 與 `unpromotedCuts`

## Impact

- Affected specs: desktop-roadmap（新）、desktop-app、client-protocol
- Affected code:
  - New:
    - packages/ui/src/components/RoadmapView.tsx
    - packages/ui/src/roadmapModel.ts
    - packages/ui/src/__tests__/roadmapModel.test.ts
    - packages/ui/src/__tests__/roadmapView.test.tsx
  - Modified:
    - apps/desktop/core/src/query.rs
    - apps/desktop/src-tauri/src/remote.rs
    - packages/ui/src/adapter.ts
    - packages/ui/src/components/KanbanBoard.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/index.ts
    - apps/desktop/src/store.ts
    - apps/desktop/src/App.tsx
    - apps/desktop/src/deepLink.ts
    - apps/desktop/src/__tests__/deepLink.test.ts
    - apps/desktop/src/__tests__/store.test.ts
    - apps/desktop/src/__tests__/App.test.tsx
    - apps/desktop/src/__tests__/remoteCapabilities.test.tsx
    - apps/desktop/src/__tests__/remoteDataSource.test.ts
    - packages/ui/src/__tests__/kanban.test.tsx
- 影響的 app：apps/desktop（Tauri 殼的清單回應與前端）、packages/ui；speclink-core、speclink-cli、server 不動
- 依賴：不新增套件；依賴線以 SVG 自行繪製

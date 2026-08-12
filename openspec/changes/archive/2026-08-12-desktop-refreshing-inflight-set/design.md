## Context

desktop-loading-skeleton-ux 用全域布林 `refreshing` 承載「整批載入進行中」，記帳分散：六個翻頁入口以必填 `willRefresh` 參數表態是否預標、refresh() 的出口以 activeKey＋世代雙守衛決定誰有權收旗標、`resetWorkspaceTransientState` 於切換時重置。該機制歷經四次記帳錯誤修補後可用，但「入口忘記標／出口忘記收」在結構上仍可能隨新路徑再現。review 票留有結構線索：refresh() 三條離開路徑重複清旗標、TraySnapshot 三欄整組穿透面板。verify 票留有行為缺口：首訪載入失敗後與真空態同貌（含 remote 修復頁不遮看板的情境）。

## Goals / Non-Goals

**Goals:**

- 「載入中」由載入動作本身導出（per-workspace 在途計數），刪光手動記帳與守衛。
- tray 載入態收斂為單一導出欄位並納入即時推送，去抖例外作廢。
- 首訪載入失敗有終態呈現，與真空態可區分（看板＋tray 面板）。

**Non-Goals:**

- 重試按鈕等新互動；remote recovery 狀態機；抽屜文件三態；server-web 接線；review 票其餘 SUGGESTION（見 proposal）。

## Decisions

**D1. 在途計數 Map<workspaceKey, number>，settle 單點遞減**
store 內部持 `inflightRefreshes: Map<string, number>`（非 React 狀態；變動時同步導出，見 D2）。refresh() 於第一個 await 前對 sourceKey 計數 +1，settle（成功、失敗、世代過期）於單一 finally 遞減、歸零即刪 key。骨架讀取端條件＝「activeKey 在途計數 > 0 且 !loaded」。同 key 重疊：兩發各自 +1/-1，先發 settle 不影響後發（取代世代守衛）；跨 workspace：key 不同互不干擾（取代 activeKey 守衛）；closeTab／切換：不需清理，計數隨各發 settle 自然歸零（快照 prune 只影響 loaded 真值，不影響在途帳）。
替代方案：布林＋繼續補守衛——每個守衛只擋已知路徑，淘汰；Set<key>——同 key 重疊時先發 settle 會誤刪，計數是 Set 的正確版本。

**D2. 導出欄位 loadingActive，翻頁入口零記帳**
AppState 的 `refreshing: boolean` 改名為導出語意 `loadingActive: boolean`（＝activeKey 的在途計數 > 0；計數變動與 activeKey 變動時由 store 內部重算後 set，讀取端不觸 Map）。`workspaceActivationState` 刪除 `willRefresh` 參數與 refreshing 欄位——翻頁入口不再寫載入態，翻頁後接的 refresh() 在同一同步段內 +1 並重算，React 同步批次下主視窗無「已翻頁、未標記」空窗（desktop-loading-skeleton-ux 的預標 hack 作廢）；不接載入的入口（開修復頁、重連 handshake）天然為 false，無需表態。App.tsx 骨架條件改 `(pendingTabKey !== null || loadingActive) && !loaded`。
替代方案：讀取端直接查 Map——Map 非 React 狀態不觸發重渲染，淘汰；把計數放進 React 狀態——每發 refresh 起訖都重渲染整棵樹兩次且序列化進 TraySnapshot，過重。

**D3. TraySnapshot 收斂為 workspaceLoading 單欄並納入即推面**
TraySnapshot 刪 `workspaceLoaded` 與 `workspaceRefreshing`，改單欄 `workspaceLoading: boolean`（＝主視窗骨架條件的 B 段部分：`loadingActive && !loaded`；A 段 spinner 沿用既有 `pendingTabKey` 欄）。面板骨架條件＝`pendingTabKey !== null || workspaceLoading`，面板不再自行組合旗標（真正薄渲染，解 Data Clumps）。即時推送面 surfaceKey 改 `[pendingTabKey, activeKey, workspaceLoading]`——失敗時 `workspaceLoading` 翻 false 即時推送，面板骨架即時收掉；desktop-loading-skeleton-ux design D5「refreshing 不進即推面」的例外作廢（本設計取代之）。推送成本：每輪整批載入起訖各一次即推，內容 payload 照舊去抖。
替代方案：三欄照舊只加 refreshing 進 surfaceKey——面板仍自行組合三欄，Data Clumps 未解，淘汰。

**D4. 首訪失敗終態＝WorkspaceSnapshot.loadFailed**
WorkspaceSnapshot 增 `loadFailed: boolean`：整批載入失敗且該發為現任世代時 set true，成功載入 set false；隨快照存續（切走再切回仍記得失敗）。呈現條件＝`!骨架條件 && !loaded && loadFailed`：看板卡片區與 tray 面板分區顯示「載入失敗，稍後自動重試或重新切換分頁」語意的提示文案（i18n 新鍵，中英皆補；KanbanBoard 增可選 `loadFailed` prop，DiscussionColumn 同步；TrayPanel 分區同款），取代原本的真空態文案。loaded 語意不變（讀不到 ≠ 確認是空的）。remote 修復頁不遮看板的情境（verify 票）自然落入此終態。
替代方案：失敗也標 loaded＝true——違反 desktop-loading-skeleton-ux D2 已定案的語意且撞既有契約測試，淘汰；放 recovery 狀態機——只覆蓋 remote，淘汰。

**D5. 施工順序與遷移**
待 desktop-loading-skeleton-ux 併回 main 後動工。遷移為原地改寫：`refreshing` 讀寫點全數改 `loadingActive` 導出；既有測試中「記帳邊界」系列（關閉在途分頁、跨 workspace 不互清、同 key 重疊、翻頁與標記同批、不接載入的入口不標）語意不變、逐條改寫斷言對象——它們正是本重構的回歸網。

## Implementation Contract

**Behavior（完成後可觀察行為）**

- 骨架的出現與消失時機與 desktop-loading-skeleton-ux 落地行為一致（首訪載入中出骨架、已訪不閃、失敗骨架收掉），既有五類記帳邊界情境全數維持。
- 新增：首訪整批載入失敗後，看板卡片區與 tray 面板分區顯示載入失敗提示文案（非空態文案、非骨架）；下一次成功載入後恢復正常呈現。tray 面板骨架於失敗當下即時收掉（不再等去抖週期）。
- 動畫與 aria 標記維持現狀（skeleton aria-busy、spinner aria-label）。

**State／Interface**

- store：刪 `refreshing`；增 `loadingActive: boolean`（導出，初值 false）；`workspaceActivationState(key)` 恢復單參數；WorkspaceSnapshot 增 `loadFailed: boolean`（初值 false）。
- tray：TraySnapshot 刪 `workspaceLoaded`／`workspaceRefreshing`、增 `workspaceLoading: boolean` 與 `workspaceLoadFailed: boolean`；surfaceKey＝`[pendingTabKey, activeKey, workspaceLoading]`。
- packages/ui：KanbanBoard／DiscussionColumn 增可選 `loadFailed?: boolean`（僅新增可選 props，不做破壞性變更）。
- i18n：新增載入失敗提示鍵（desktop messages；packages/ui i18n 如看板文案落於該處則同步）。

**Verification**

- 單元測試（vitest）：在途計數的五類記帳邊界（沿用既有測試改寫斷言對象）＋同 key 重疊雙發起訖；loadFailed 的設定／清除／現任世代守衛；TraySnapshot 單欄導出與失敗即推；看板與面板的失敗終態渲染（失敗提示、非空態、非骨架）。
- 既有測試全綠：apps/desktop 與 packages/ui 的 vitest 套件。
- 手動驗收（[M] 任務）：斷網或以無效 remote 首訪 workspace，確認失敗提示出現且不與空 workspace 混淆；恢復後自動或重切分頁回復正常。

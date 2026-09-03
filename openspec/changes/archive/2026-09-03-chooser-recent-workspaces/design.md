## Context

桌面 app（apps/desktop，React＋zustand＋Tauri 殼）的 workspace 記憶只有一份：分頁列。`apps/desktop/src/tabs.ts` 以 localStorage 鍵 `speclink.projectTabs` 持久化分頁（v2：version＋locator＋顯示名＋activeKey），上限 10，關閉分頁即自清單移除；檔頭注解明寫「分頁列即最近開啟清單」。「新增 Workspace」chooser（`apps/desktop/src/components/WorkspaceChooser.tsx`）第一步只畫「本機資料夾」與「Speclink Server」兩張來源卡。

分頁身分是 WorkspaceLocator（`apps/desktop/src/session.ts`：local 帶 root；remote 帶 connectionId／projectId／repoId／可選 checkoutRoot），比對一律經 locatorKey。成功開啟的尾聲集中在 store 三個閉包函式：本機的 enterProject、remote 的 adoptRemoteSession、本機轉 remote 的 replaceLocalWorkspaceWithRemote；啟動還原在 restoreTabs。remote session 的顯示名為「projectName/repoName」。

2026-09-02 討論 chooser-recent-workspaces 裁定：分頁列只回答「現在開著什麼」，另設一份「以前開過什麼」的記憶，形態如 VS Code 的最近開啟；儲存方式授權由本設計決定。本變更全部落在 apps/desktop 前端：不新增 Tauri command、不動 apps/desktop/core 與任何 Rust crate、不動 CLI 與設定欄位，因此 crate 邊界規則（core／fs／store／host）不適用，desktop 純邏輯歸 TypeScript 純函式模組並以 vitest 直測。

## Goals / Non-Goals

**Goals:**

- chooser 第一步列出曾成功開啟過、但目前不在分頁列上的 workspace，本機與 remote 都列。
- 記錄與分頁列分離：關閉分頁不動記錄；記錄以 localStorage 獨立鍵持久化，不寫入任何專案目錄。
- 點項目沿既有開啟流程，不繞過探測與 init 確認；失效項目轉錯誤態且可移除。
- 升級後第一次啟動以既有分頁補種，清單不會一開始是空的。
- 既有分頁行為（去重、上限 10、關閉移除、錯誤態、鍵盤切換）與分頁持久化 v2 格式零改動。

**Non-Goals:**

- 零分頁的空狀態引導頁與系統匣選單不列最近開啟（同一鍵可直接讀，另案）。
- 清單不跨機器、不在重裝或清除 WebView 資料後保留（與分頁同一耐久等級）。
- 不記錄開啟時間、不顯示時間戳（順序已表達新舊）。
- 不提供「清除全部」；每筆可移除已足夠。
- 不把清單塞進分頁持久化鍵（會逼出 v3 遷移並把兩個概念綁死），不走 Rust 側 app_config_dir 的 JSON 檔（要新增讀寫 command 卻沒有第二個讀者）——皆為討論已否決的方案。

## Decisions

### D1 最近開啟清單模組

新增純函式模組 `apps/desktop/src/recents.ts`，形態與 tabs.ts 對稱：條目 `RecentEntry = { locator: WorkspaceLocator; name: string }`，常數 `MAX_RECENTS = 20`，localStorage 鍵 `speclink.recentWorkspaces`，payload `{ version: 1, entries: RecentEntry[] }`，entries 最新在前。函式：`upsertRecent(entries, entry)`（同 locator key 去重後放到最前、超過上限截尾）、`removeRecent(entries, key)`、`persistRecents(entries, storage = localStorage)`、`readPersistedRecents(storage = localStorage)`（鍵缺席回 null；壞 JSON、version 不符或形狀不識別回空陣列；逐條以 locator 形狀驗證丟棄壞條目）、`visibleRecents(entries, tabs)`（見 D3）。tabs.ts 的私有 `isLocator` 改為匯出供 recents 共用，不複製一份。

為什麼獨立鍵而非併入 `speclink.projectTabs`：分頁鍵已有 v1→v2 遷移邏輯，再加欄位就是 v3；兩個概念各自歸零、各自演進更簡單。為什麼 localStorage 而非 Rust 側檔案：與分頁同一耐久等級即符合需求，零 IPC、零 Tauri command，唯一讀者是 chooser。為什麼最新在前而非像分頁 append 尾端：清單的消費者只看「最近」，最新在前讓截尾與呈現都不用反轉。

### D2 成功開啟即記入

記錄集中在一個閉包函式 `recordRecent(entry, dropKey?)`：算出「記到最前」的新清單並回傳，本身不落地；`dropKey` 供本機資料夾轉 remote 的路徑移除同一資料夾的 local 條目（同一資料夾不留兩筆）。持久化由呼叫端在自己的 `set()` 之後、緊挨著 `persistTabs` 做——若在函式內就寫檔，寫檔與狀態更新之間的任何一步拋錯（如 enterProject 的 `createSession`）都會留下「已寫檔、狀態沒進去」的分歧。呼叫點是 store 的每一個「成功開啟尾聲」：

- enterProject（本機開啟，含分頁點擊走的探測路徑）
- adoptRemoteSession（remote 採納，含 chooser scope 流程、remote marker 探測、本機與 remote 並存衝突的「以 server 為準」）；它接受 `dropRecentKey`，marker 分流經 `openRemoteWorkspace` 的第四參數 `dropLocalRoot` 傳入
- reconnectRemoteTab（remote 分頁重連成功）與 activateTab 的 remote 既有 session 分支——remote 分頁點擊同樣是一次成功開啟，順序不得停在舊值
- replaceLocalWorkspaceWithRemote（正式遷移，帶 local key 移除舊條目）

關閉分頁（removeTab 的呼叫端）與分頁上限淘汰都不動 recents。store 動作名為 `forgetRecent(key)`（與模組函式 `removeRecent` 分名，避免別名 import）：自狀態與 localStorage 移除該筆。

為什麼不在 upsertTab 內順手記：upsertTab 是純函式且分頁淘汰語意（丟最舊）與 recents（丟第 21 筆）不同，混在一起會讓兩份測試互相牽制。restoreTabs 的啟動還原不另外記入：背景分頁只探測路徑、不進尾聲；活躍分頁的還原走 enterProject／remote 採納的同一尾聲，只會把它移到最前而不新增條目（這些條目本就在清單裡，或由 D5 補種）。

### D3 顯示期過濾

`visibleRecents(entries, tabs)` 回傳 locator key 不在分頁列上的條目，順序不變。過濾在 App 接線層做（`apps/desktop/src/App.tsx` 把 `visibleRecents(s.recents, s.tabs)` 傳給 chooser），chooser 只負責畫。過濾後為空時 chooser 不畫「最近開啟」區段（連標題都不畫）。記錄本身不刪已開著的條目：分頁一關，該條目自然回到清單。

為什麼不在寫入時就排除已開著的：那會讓「關掉分頁後找得回來」失效，違背記憶的目的。

### D4 點擊開啟與失效錯誤態

chooser 新增 props：`recents: RecentEntry[]`、`onRemoveRecent(key)`；開啟沿既有 `onOpenLocal`／`onOpenRemote`，不新增 store 動作。區段畫在 source 步驟、兩張來源卡下方、且僅在未進入 localProject 子畫面時顯示；每列是一個開啟按鈕加一個移除按鈕（兩個相鄰按鈕，不巢狀），列上有圖示（本機 Folder、remote Cloud、錯誤態 AlertTriangle）、顯示名、副標（本機為 root 路徑、等寬字；remote 為「連線名稱 / 顯示名」，連線名稱自既有 `connections` prop 以 connectionId 查得）。

點本機項目：先 `workspace.openProject(root)` 探測（與 chooseLocal 同一步驟）；探測拋錯（路徑不存在或不可讀）即把該筆標為錯誤態並顯示錯誤字串，不呼叫 onOpenLocal；探測成功即關閉 chooser 並呼叫 `onOpenLocal(root)`——後續的專案／未初始化／remote marker 分流全由既有 openProjectAt 承擔，本機不直接跳 localProject 子畫面（最近開啟的意圖是「直接開」，與首次選資料夾不同）。

點 remote 項目：帶 checkoutRoot 的條目先 `inspectCheckout(checkoutRoot, origin, projectId, repoId)` 驗資料夾仍與該 scope 一致，與其他 checkout 開啟路徑同一步——handshake 只問伺服器，不會發現資料夾已消失；驗證拋錯即標為錯誤態。無 checkout 綁定的條目（規格模式）跳過這一步。之後呼叫 `onOpenRemote(connectionId, "projectId/repoId", checkoutRoot)`（target 格式與 store 重連邏輯一致），handshake 拋錯即標為錯誤態並顯示錯誤字串。

連線狀態的判定等連線清單「讀取成功」才做，不是「載入結束」才做：`refreshConnections` 把讀取失敗吞掉並保留現值（初值空陣列），只看是否 settle 會讓讀取失敗與「真的沒有連線」無從分辨，把有效條目誤標成已移除。因此 `refreshConnections` 改回傳 `Promise<boolean>`，chooser 只在它回 true 時判定。判定分兩種原因——connectionId 不在清單內為「連線已移除」，在清單內但未登入為「連線已登出」，兩者都停用開啟按鈕、保留移除按鈕。清單未就緒時列維持可用。此時點下去，只有綁著 checkout 的條目會在 openRecent 內補判並落同一句純文字理由（不拋錯，避免訊息帶 `Error:` 前綴）——那條路徑需要 connection 的 origin 才能驗 checkout；規格模式（無 checkout 綁定）只需 connectionId，照常執行 remote 開啟，不得因清單未就緒而被擋下。錯誤態只活在 chooser 的元件狀態（`recentErrors: Record<key, string>`），chooser 重新開啟即清空；錯誤態的列開啟按鈕停用、移除按鈕照常可用。移除呼叫 `onRemoveRecent(key)`。

為什麼失效偵測採「點擊時」而非「開啟 chooser 時逐筆探測」：20 筆各打一次 IPC 只為畫圖示不划算；點擊才探測與分頁列「點擊才知道失效」的既有語意一致，remote 連線缺席則零成本可在畫面上直接判定。

### D5 升級補種

restoreTabs 讀分頁後接著 `readPersistedRecents()`：回 null（鍵缺席）時以持久化分頁補種——分頁順序是 append（最後一個最新），補種時反轉為最新在前——並立刻 persistRecents；回陣列（含空陣列與壞資料歸零）時照用、不補種。補種只發生在鍵缺席這一次，之後使用者清空清單不會被再次補回。

### D6 分頁列不再是最近清單

正典 desktop-config 的「專案分頁列存於 app 本機」需求改寫為「分頁列呈現目前開著的專案」，並明示關閉分頁自分頁持久化清單移除但不影響最近開啟清單；五個 scenario 原封保留。程式碼側只改 tabs.ts 檔頭注解（「分頁列即最近開啟清單」改為「分頁列只管目前開著的；最近開啟另見 recents.ts」），分頁邏輯零改動。

## Implementation Contract

**可觀察行為**

- 使用者依序開啟 A、B，關閉 B 的分頁，再按「新增 Workspace」：第一步兩張來源卡下方出現「最近開啟」區段，列出 B（不列 A，因 A 仍開著）。點 B 的列：chooser 關閉、B 的分頁回到分頁列並成為活躍分頁，行為與經「本機資料夾」選同一路徑一致。
- 所有記錄都在分頁列上時，「最近開啟」區段整段不顯示。
- 同一 workspace 重複開啟只留一筆並移到最前；清單超過 20 筆時丟最舊。
- B 的資料夾被刪除後點 B 的列：該列轉錯誤態並顯示錯誤原因，不建立分頁、不切換專案；列上的移除鈕可把它自清單移除，之後重啟 app 也不再出現。
- remote 項目的連線已自「伺服器」頁移除時：該列直接以錯誤態呈現「連線已移除」，開啟鈕停用，可移除。
- 升級到本版第一次啟動（localStorage 無 `speclink.recentWorkspaces` 鍵）：清單自既有分頁補種，最新的分頁在最前。
- 文案：zh-TW「最近開啟」／「自最近開啟移除」／「連線已移除」／「連線已登出」；en「Recently opened」／「Remove from recently opened」／「Connection removed」／「Connection signed out」。ja 不在 app 支援的介面語言內（desktop-config 只支援 zh-TW 與 en）。

**介面與資料形狀**

- localStorage 鍵 `speclink.recentWorkspaces`，值為 JSON `{ "version": 1, "entries": [{ "locator": WorkspaceLocator, "name": string }] }`，entries 最新在前、最多 20 筆。既有鍵 `speclink.projectTabs` 的格式與內容不變。
- `apps/desktop/src/recents.ts` 匯出：`MAX_RECENTS`、`RECENTS_STORAGE_KEY`、`RecentEntry`、`upsertRecent`、`removeRecent`、`persistRecents`、`readPersistedRecents`、`visibleRecents`；`apps/desktop/src/tabs.ts` 額外匯出 `isLocator`。
- store 狀態 `recents: RecentEntry[]`（初始空陣列）與動作 `forgetRecent(key: string): void`；`openRemoteWorkspace` 多一個選填的第四參數 `dropLocalRoot`。
- WorkspaceChooser props 新增 `recents: RecentEntry[]`（已過濾）與 `onRemoveRecent: (key: string) => void`；開啟沿用既有 `onOpenLocal`／`onOpenRemote`／`workspace.openProject`。
- i18n 鍵：`chooser.recentTitle`、`chooser.recentRemove`、`chooser.recentConnectionMissing`、`chooser.recentConnectionLoggedOut`，zh-TW 與 en 同鍵。
- `onRefreshConnections`（store 的 `refreshConnections`）回傳 `Promise<boolean>`——讀取是否成功；chooser 據此區分「清單真的空」與「讀不到」。

**失敗模式**

- localStorage 鍵存壞 JSON、version 不是 1、entries 不是陣列：讀成空陣列（不補種），app 照常啟動，下一次成功開啟寫回合法 v1 payload。條目 locator 形狀不識別或 name 非字串：丟該條目、其餘保留。讀取時同時收斂手改或舊版殘留的 payload——同 locator key 只留第一筆，超過 20 筆截尾。
- 本機探測失敗與 remote handshake 失敗：錯誤字串顯示在該列下方，chooser 保持開啟；記錄不自動刪除（由使用者移除）。
- localStorage 寫入拋錯（配額或私密模式）：與 persistTabs 同等處理，不攔截、不彈窗。

**驗收**

- `npm test -w @speclink/desktop` 全綠，含新增的 `apps/desktop/src/__tests__/recents.test.ts`、擴充的 `store.test.ts`、`workspaceChooser.test.tsx`、`tabs.test.ts`；`messages.test.ts` 的 zh-TW／en 鍵集對稱檢查通過。
- 手動：依「可觀察行為」第一、三、四條在 tauri dev 實機各走一次。

**範圍邊界**

- 範圍內：recents.ts、store 的 recordRecent 與其每個成功尾聲呼叫點、restoreTabs 補種、forgetRecent、chooser source 步驟的區段與錯誤態、App 接線、i18n 兩語系、tabs.ts 注解與 isLocator 匯出、對應測試、desktop-config 與 workspace-chooser 規格。
- 範圍外：空狀態引導頁、系統匣、Rust／Tauri 任何檔案、CLI、docs、分頁列行為與其持久化格式。

## Risks / Trade-offs

- [回歸對照] 本變更不動 CLI 與引擎，golden 與 CLI 整合測試無涉；前端回歸面是 `npm test -w @speclink/desktop`，其中 workspaceChooser.test.tsx 既有案例對 source 步驟以角色與文字定位按鈕，新增區段不得改動既有兩張來源卡的可及性名稱 → 新區段的按鈕 aria-label 用「最近開啟」前綴，不與「本機資料夾」「Speclink Server」重名。
- [跨平台] 本機路徑只作為 locator key 與副標顯示，不做路徑比對以外的處理；Windows 反斜線與 macOS／Linux 正斜線都原樣存取，locatorKey 已是既有比對基準 → 不新增任何路徑正規化。
- [同一資料夾兩筆] 本機轉 remote 後若不移除 local 條目，清單會出現同一資料夾的 local 與 remote 兩筆 → D2 在 replaceLocalWorkspaceWithRemote 明確移除 local 條目。
- [連線改名] remote 列的連線名稱自 connections 即時查得而非寫死進記錄，改名即反映；連線被移除則轉錯誤態 → 記錄只存 connectionId，不存名稱。
- [清單與分頁不同步] 分頁列的上限淘汰（第 11 個分頁擠掉最舊）不動記錄，被擠掉的 workspace 仍在最近開啟 → 這正是記憶的用途，屬預期行為。
- [補種只一次] 使用者手動清空清單後不會再補種；壞資料歸零也不補種 → 兩者皆為刻意，避免「刪了又長回來」。

## Migration Plan

- 部署：隨桌面 app 版本發布；首次啟動由 D5 自既有分頁補種，使用者無需操作。
- 回滾：退回舊版後 `speclink.recentWorkspaces` 鍵成為無人讀取的殘留資料，對舊版無影響；再升級時鍵已存在、不再補種。

## Open Questions

無。

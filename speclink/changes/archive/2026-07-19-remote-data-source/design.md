## Context

已就位：session 模型（remote locator 型別已宣告、無建構路徑）、connection registry 與 Keychain credential（refresh rotation）、speclink-remote typed client（handshake、changes/artifacts/tasks/claim/archive/discussions/context、/sync-state 未包但 server 有）、server 的 /events SSE（eventId＝outbox seq、Last-Event-ID resume、reset 信號）與 /sync-state ETag。缺口：client 端無 SSE 消費者；SpeclinkDataSource 21 方法對 server 端點的覆蓋不完整（archived 瀏覽、searchWorkspace、正典 spec 內文、validate/analyze、deleteChange、moveTask、reorderCard 無端點；使用者可及的 Project/Repo 清單端點也不存在——影響開啟入口形狀）。桌面 UI 一律經 SpeclinkDataSource，remote 化的正確切點就是它。

## Goals / Non-Goals

**Goals:**

- remote session 可被真正開出：handshake fail-closed → 分頁 → 看板/文件/任務/動詞/討論可用。
- roadmap Phase 3 gate 兩條落地：capability 驅動停用；SSE 中斷以 Polling＋ETag 恢復。
- token 換發與所有 secret 流動維持 Rust 側；TS 只見狀態。

**Non-Goals:**

- server 零改動：端點缺口以停用如實呈現；補讀取端點（archived、spec 內文、search、專案清單）屬後續獨立 server 刀。
- 無完整 Workspace chooser（刀 4）、無 checkout 綁定與完整 drift/verify（RD 面後續刀）、無離線 snapshot 唯讀與重新認證完整 UX（刀 6——本刀只回報錯誤狀態）。
- 本地 session 行為零改動；不做多 server 同時多連線之外的最佳化（連線本身天然 per-server）。

## Decisions

### 決策 1：RemoteDataSource 的三類覆蓋矩陣

以 SpeclinkDataSource 為準逐方法定案：(a) 直達——listChanges、listSpecs、status（get_change）、getDocument（get_artifact）、setTaskDone（done/undone）、runVerb 之 archive、listDiscussions、getDiscussionDocument（show_discussion）、promoteDiscussion、archiveDiscussion、changeMeta 與 changeCapabilities（自 get_change 與清單 payload 能取則取、否則歸入 (c)——實作時以 server 實際 payload 定奪並記入 capability 描述）；(b) 組合——setAllTasks 以逐任務 done/undone 迴圈組合（非原子、逐筆失敗即中止並回報）；(c) 明確不支援——listArchived、getArchivedDocument、archivedCapabilities、searchWorkspace、getSpecDocument、runVerb 之 validate/analyze、deleteChange、moveTask、reorderCard。(c) 類回拒絕錯誤且 capability 描述標記為不支援，UI 據此停用。凍結原則：server 缺什麼就停用什麼，不在 client 端偽造。

### 決策 2：capability 描述隨 session 建立產生

RemoteDataSource 建立時附帶 capability 描述物件（逐操作布林），來源＝binding handshake 回應＋決策 1 矩陣常量；UI 消費它停用 affordance：刪除與驗證動詞按鈕 disabled 附繁中 tooltip、看板搜尋輸入 disabled 附繁中 tooltip、看板與任務拖排整段不掛（把手不渲染——絕不留下點了沒事的假 affordance；全勾＝setAllTasks 屬 (b) 組合實作、照常可用）；archived 頁與 spec 內文區顯示「此 server 尚未提供……」提示卡。本地 session 的 capability 描述全真——同一 UI 路徑、零分岐維護。停用所需的 packages/ui 改動一律為附加性 optional props（未傳＝行為與現狀逐位元相同），既有測試零修改全綠。

### 決策 3：SSE 消費落 speclink-remote、event manager 落 src-tauri

speclink-remote 新增 events 模組：以既有 HTTP 層對 /events 開流、逐行解析 SSE（id/event/data）、暴露 typed 事件（invalidate 提示含 scope 與 resource、reset）、支援 Last-Event-ID 請求頭與可中止的阻塞讀取。event manager（src-tauri）持 per-connection 單一訂閱執行緒：收 invalidate → 以 Tauri 事件 remote-workspace-changed（payload＝locator key）通知前端；連線失敗/中斷 → 進收斂程序（決策 5）。多 session 同 connection 共用同一條流——manager 以 connection 為鍵、以 session 註冊表分發。

### 決策 4：token 生命週期與 401 語意

per-connection TokenManager（src-tauri）：access token 記憶體持有；請求前無 token 或已知過期即以 Keychain refresh credential 換發（rotation 新 rt 立即回寫 Keychain——刀 2 既有語意）；任何請求 401 → refresh 一次 → 重試一次 → 仍失敗即令該 connection 進入 needs-reauth 狀態（TS 可見布林＋繁中訊息，session 操作回拒絕錯誤）——完整重新認證 UX 屬刀 6。SSE 流的 401 同語意。所有 token 僅存在 Rust；TS 面只有狀態。

### 決策 5：斷線收斂程序（§9.2 的桌面實體）

SSE 中斷或 Last-Event-ID 續傳被 server 以 reset 拒絕時：(1) 停流；(2) 對 /sync-state 取 ETag 與現值比對——不同即發 remote-workspace-changed 令前端重載（Query 為重讀正典）；(3) 以指數退避重連 SSE（成功續傳則正常路徑、收 reset 信號則發全量重載通知後從新 eventId 起訂）。輪詢僅在 SSE 不可用期間作為心跳（間隔常數化、可測注入時鐘不引入——以可注入的退避序列測試）。push 永不攜帶資料實體、只做 invalidate。

### 決策 6：極簡開啟入口與 handshake fail-closed

伺服器頁籤已登入條目新增「開啟 workspace」：極簡對話輸入 workspace 識別——`projectKey` 或 `projectKey/repoKey`（project key 構成 project-scoped URL 路徑、斜線後半為 X-Speclink-Repo 值；省略 repo 時由 server 裁定：單 repo 自動綁定、多 repo 回多義拒絕。使用者可及的專案清單端點不存在——文字輸入是唯一誠實形狀，完整 chooser 與清單端點屬後續刀）→ src-tauri 以該 repo handshake → 成功回 project/repo 顯示名與 capability，建立 remote session 與分頁（locator kind:"remote"，connectionId＋handshake 回的 project/repo 識別；checkoutRoot 缺席）→ 失敗（403/404/多義）原樣呈現 server 錯誤、不建分頁。remote 分頁顯示 cloud 圖示與 Project/Repo 名；持久化 v2 天然承載 remote locator，重啟後 remote 分頁恢復（重建 session 需 handshake 重走，失敗顯示 needs-reauth/錯誤狀態而非靜默消失）。

### 決策 7：TS RemoteDataSource＝薄 invoke 包裝

apps/desktop/src/adapter/remoteDataSource.ts 對每個 SpeclinkDataSource 方法呼叫對應的 remote_* Tauri command（參數帶 connectionId＋repo）；所有 HTTP、token、重試邏輯在 Rust。createLocalSession 之外新增 createRemoteSession（session.ts 的工廠面擴充）；store 對 session 的消費零改動——這正是刀 1 立 seam 的回報。

## Implementation Contract

- 自動測試（in-process speclink-server＋memory identity，沿刀 2 模式）：handshake 失敗不建 session；三類矩陣逐方法（直達回真值、組合的中止語意、不支援回拒絕）；401→refresh→重試→needs-reauth 全鏈；SSE invalidate→前端事件 payload 帶 locator key；斷線→ETag 收斂→Last-Event-ID 續訂；reset→全量重載通知；同 connection 兩 session 共用單流（訂閱計數）。TS：RemoteDataSource 方法對 invoke 的參數映射、capability 停用的 UI 呈現（假 adapter）。
- GUI 鐵律（真實視窗；操作前確認使用者未在使用螢幕）：npm run dev 起本地 server → 已登入連線「開啟 workspace」輸入 repo → remote 分頁出現、看板呈現 server 上的 changes/discussions → 於 server 側以 CLI 建 change 後看板數秒內反映（SSE invalidate）→ 手動重啟 server 程序驗證斷線收斂與續訂 → 不支援操作（刪除、搜尋、archived 頁）呈現停用/提示 → 重啟 app remote 分頁恢復。
- 回歸：cargo test -p speclink-remote、npm test -w apps/desktop、npm test -w packages/ui（介面零改動）、cargo build --release -p speclink-desktop 全綠。
- 邊界：SSE 不可用（如代理剝流）時純輪詢路徑仍收斂；/sync-state 404（舊 server）視為無事件能力、退化為手動重整並記入 capability 描述；組合實作 setAllTasks 中途失敗回報已完成筆數。

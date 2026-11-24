## Context

Rust 端的 `TokenManager`（`apps/desktop/src-tauri/src/remote.rs`）每個 Server origin 一顆，由 `ConnectionsState::manager_for_connection` 惰性建立並綁上 connectionId。它記錄連線健康狀態（online、offline、needs-reauth）與連續失敗次數：`execute` 每次請求後，連不上或 500 以上算一次失敗，連續 3 次（`DEFAULT_FAILURE_THRESHOLD`）轉 offline；任何成功或低於 500 的回應歸零並回 online；401 走換發，仍 401 轉 needs-reauth。狀態轉換時經 `state_observer` 以 app 層級事件 `remote-connection-state` 廣播。offline 只擋寫入，讀取請求照樣會送出。

前端的缺口有三個：

- 沒有人替沒點過的遠端分頁送請求。啟動時 `restoreTabs` 只對背景本機分頁呼叫 `projectStats`；背景輪詢（`remote_watch`）只由作用中分頁的 `session.events.subscribe` 啟動。
- `remote-connection-state` 只在作用中 session 的 `subscribe` 裡被聽到，而且以該 session 的 connectionId 過濾；作用中分頁是本機專案時，沒有人在聽。
- `ProjectRail` 對「沒有 session 也沒有復原狀態」的遠端分頁一律當 ready；`remoteWorkspaceStatus` 在沒有 session 時回 error（系統匣直接用這個結果）。

## Goals / Non-Goals

**Goals:**

- 有遠端分頁的每台 Server，在 app 啟動、主視窗取得焦點與每 60 秒時被檢查一次；連不上時數秒內轉 offline。
- 沒點過、以及點過但在背景的遠端分頁，在圖示列與系統匣呈現所屬 connection 的 offline 或 needs-reauth。
- 連線狀態仍只由 Rust 判定；前端只負責觸發檢查與呈現。

**Non-Goals:**

- Server 恢復後不自動 handshake，也不替背景分頁建立 session。
- 不偵測單一 repo 的問題（存取被拒、repo 被刪）：那是分頁層級的狀態，仍由作用中分頁的輪詢判定。
- 不改點下去之後的流程：琥珀點的分頁點下去後握手失敗，照舊進復原頁並轉紅點（來源討論列為之後再決定）。
- 不改系統匣對「沒有 session、connection 也正常」的遠端分頁的既有呈現（`remoteWorkspaceStatus` 照舊回 error）；這次只在 connection 為 offline 或 needs-reauth 時改回傳該狀態。
- 檢查間隔不開放設定。
- 不移除作用中 session 既有的連線狀態轉交（它還負責回到 online 時重新整理資料）。

## Decisions

### 計時在前端、判定在 Rust 的連線檢查命令

新增 Tauri command `remote_check_connection(connectionId)`：查 connections registry 取得 origin，取得該 origin 的 `TokenManager`，做一次連線檢查後回傳這條 connection 的目前狀態。何時檢查由前端決定：前端本來就持有分頁列、主視窗焦點事件與 app 生命週期，Rust 端不知道哪些 connection 有分頁。

替代方案：Rust 端每個 origin 開一條常駐執行緒自己計時 — 前端仍要把「哪些 connection 有分頁」同步給 Rust，還要處理執行緒的建立與收束，程式多一倍而行為相同。啟動時逐分頁握手 — 來源討論已排除（N 個分頁 N 次完整連線，而且只檢查一次）。

### 檢查請求用身分查詢

檢查經 `TokenManager::execute` 呼叫 `speclink_remote::device::identity`：先問 `/api/speclink/v1/whoami`，Server 回 404 或 405 時改問根層 `/auth/whoami`，與登入時查身分的做法相同。這個請求不需要專案、repo 或版本標頭，回應很小，而且需要有效身分，所以同時能發現 needs-reauth。

### 失敗後快速重試到閾值

`TokenManager` 新增方法 `check_connection(credentials, call, retry_delays)`：

1. connection 為 needs-reauth 時不送請求，直接結束。
2. 以 `execute` 送一次請求。成功即結束。
3. 失敗時：錯誤不計入離線（Server 有回應，例如 403、404）即結束；connection 已不是 online（剛轉 offline，或轉 needs-reauth）即結束；否則等 `retry_delays` 的下一個間隔再送，間隔用完即結束。

`lib.rs` 的重試間隔常數為 1 秒、2 秒，最多送 3 次，等於離線閾值（間隔個數以編譯期檢查綁定 `DEFAULT_FAILURE_THRESHOLD`），所以 Server 停著時約 3 秒內轉 offline。connection 已經是 offline 時只送一次：成功就回 online，失敗就維持 offline。方法內不自行改狀態，狀態一律由 `execute` 的既有判定改寫並廣播。`TokenManager` 另記下進入目前狀態時廣播的訊息，`current_state(connectionId)` 以事件的形狀回傳目前狀態與這則訊息，命令檢查完以它回傳。測試以空間隔呼叫，避免真的等待。

這個方法放在 `remote.rs` 而不是 `speclink-desktop-core`：`TokenManager` 與它的測試都在 `remote.rs`，新方法只是它的狀態機上多一個入口；Tauri command 只做 origin 查找與委派。

### 前端每條連線一份狀態並全域監聽

store 新增 `connectionStates: Record<connectionId, RemoteConnectionStateEvent>` 與 action `applyConnectionStateEvent(event)`：更新這份對照表，並呼叫既有的 `applyRemoteConnectionState(event)` 把狀態套到所有同 connection 的 session。`ConnectionsAdapter` 新增 `onConnectionState(listener)`，以 `listen("remote-connection-state")` 訂閱並沿用 `session.ts` 既有的 `isRemoteConnectionStateEvent` 檢查 payload；`App.tsx` 在掛載時訂閱、卸載時解除。

事件只在狀態轉換時發出：webview 重建（重載、視窗重開）後前端的對照表是空的，而 Rust 端的狀態沒有再轉換，就不會再有事件。所以 `checkConnection` 回傳的目前狀態同樣交給 `applyConnectionStateEvent`，補回這種狀態。`applyConnectionStateEvent` 與 session 層的 `applyRemoteConnectionState` 在狀態與訊息都沒變時不更新，定期檢查不會讓畫面每 60 秒重繪。登出或移除連線時 Rust 端丟掉該 origin 的 `TokenManager`，新的一顆從 online 開始、重新登入也不一定有事件，所以 store 一併刪掉那條 connection 的狀態。

### 沒有 session 的分頁讀 connection 狀態

`remoteWorkspaceStatus(session, recovery, connectionState?)` 新增第三個選填參數，判斷順序為：有 recovery 回 recovery 的狀態；沒有 session 時，connectionState 為 offline 或 needs-reauth 回該狀態，否則照舊回 error；有 session 時照舊。

- `ProjectRail`：新增 prop `connectionStates`。分頁有 recovery、有 session，或所屬 connection 為 offline 或 needs-reauth 時呼叫 `remoteWorkspaceStatus`，其他情況照舊當 ready。琥珀點與提示文字沿用既有的 offline、needs-reauth 分支。
- `buildTraySnapshot`：傳入 `state.connectionStates[connectionId]`；`failureKind` 在沒有 session 且 connection 為 needs-reauth 時也回 `needs-reauth`。

### 沒有資料的離線分頁不提最後資料

既有的離線文字是「離線，顯示最後資料」（圖示列，`remote.recovery.offlineShort`）與「離線（最後資料）」（系統匣，`tray.recovery.offline`），說的是有 session、保留了最後一份資料的分頁。沒有 session 的分頁沒有資料，沿用會誤導，所以新增兩個鍵，zh-TW 與 en 各一份：

- `remote.recovery.offlineNoDataShort`：zh-TW「離線」，en「Offline」。`ProjectRail` 在狀態為 offline 且分頁沒有 session 時使用。
- `tray.recovery.offlineNoData`：zh-TW「離線」，en「Offline」。`TrayTabSnapshot` 新增 `hasSession: boolean`（遠端分頁有 session 時為 true），`tabStatusLabel` 在狀態為 offline 且 `hasSession` 為 false 時使用。

needs-reauth 沿用既有文字（圖示列「需要登入」、系統匣「需要重新登入」），它和有沒有資料無關。

### 檢查時機與同一條連線不重疊

store 新增兩個 action：

- `checkRemoteConnections()`：取分頁列上所有遠端分頁的 connectionId 去重；正在檢查中的 connection 略過；其餘各呼叫一次 `connectionsAdapter.checkConnection`，把回傳的目前狀態交給 `applyConnectionStateEvent`，完成或失敗後移出「檢查中」集合。命令的錯誤（例如連線已被移除）不呈現：狀態由 Rust 的判定決定。沒有遠端分頁時不呼叫。
- `startConnectionChecks()`：以 `setInterval` 每 60 秒（store 的模組常數，不開放設定）呼叫 `checkRemoteConnections`，回傳停止函式。

觸發點：`restoreTabs` 在寫回分頁列之後、切回作用中分頁之前呼叫一次 `checkRemoteConnections`（不等待）；`App.tsx` 掛載時呼叫 `startConnectionChecks`、卸載時停止；`App.tsx` 既有的主視窗焦點訂閱在取得焦點時也呼叫 `checkRemoteConnections`。

## Implementation Contract

- **行為**：
  - 圖示列上有遠端分頁的 Server 停止時，app 啟動約 3 秒內，所有沒有 session 的遠端分頁方塊亮琥珀點，提示附「離線」（不提最後資料）；已有 session 但在背景的分頁同樣亮琥珀點，提示照舊為「離線，顯示最後資料」。
  - Server 恢復後，下一次檢查（最遲 60 秒，或主視窗取得焦點時）成功，琥珀點消失；不發出 handshake，不建立 session。
  - connection 的登入失效（身分查詢 401 且換發失敗）時，分頁亮琥珀點，提示附既有的需要登入文字。
  - 系統匣面板的專案 tab 與原生選單的專案項，對同一個分頁呈現同一個狀態。
  - 圖示列只有本機分頁時，不送出任何檢查請求。
- **介面與資料形狀**：
  - Tauri command `remote_check_connection`，參數 `{ connectionId: string }`，成功回這條 connection 檢查後的目前狀態 `{ connectionId, state, message }`（與 `remote-connection-state` 事件的 payload 同形，message 是進入該狀態時廣播的那一則）；connection 不在 registry 時回錯誤字串「連線不存在——請重新設定 server 連線」（沿用 `connection_origin`）。
  - `ConnectionsAdapter.checkConnection(connectionId: string): Promise<RemoteConnectionStateEvent>`；`ConnectionsAdapter.onConnectionState(listener: (event: RemoteConnectionStateEvent) => void): Promise<() => void>`。
  - store 新增 `connectionStates`、`applyConnectionStateEvent`、`checkRemoteConnections`、`startConnectionChecks`。
  - `remoteWorkspaceStatus` 新增選填第三參數；既有兩參數呼叫的結果不變。
  - `TrayTabSnapshot` 新增 `hasSession: boolean`；i18n 新增 `remote.recovery.offlineNoDataShort` 與 `tray.recovery.offlineNoData`（zh-TW、en）。
  - `remote-connection-state` 事件 payload（`connectionId`、`state`、`message`）不變。
- **失敗模式**：
  - 檢查請求失敗不彈任何訊息；呈現只隨 Rust 判定的狀態改變（狀態轉換的事件，或檢查命令回傳的目前狀態）。
  - `checkConnection` 被拒（connection 已移除、非 Tauri 環境）時靜默略過。
  - 單次失敗、數秒內重試成功時，connection 維持 online，沒有事件、沒有琥珀點。
- **驗收**：
  - `apps/desktop/src-tauri/src/remote.rs` 的 `TokenManager` 測試：連續失敗時送 3 次後轉 offline 並廣播一次；第 2 次成功時只送 2 次、維持 online、沒有廣播；已是 offline 時送 1 次，成功即廣播 online，仍連不上則維持 offline、沒有廣播；needs-reauth 時不呼叫請求；回 403 時只送 1 次；檢查後的目前狀態帶著進入該狀態時廣播的訊息。以 `cargo test -p speclink-desktop` 執行。
  - `apps/desktop/src/__tests__/connectionCheck.test.ts`：`checkRemoteConnections` 對兩個同 connection 的遠端分頁與一個本機分頁只呼叫一次 `checkConnection`；只有本機分頁時不呼叫；同一 connection 在檢查中時不重複呼叫；`startConnectionChecks` 以假計時器推進 60 秒後再呼叫一次、停止後不再呼叫；`restoreTabs` 會觸發一次檢查；`applyConnectionStateEvent` 更新 `connectionStates` 並把同 connection 的 session 標為 offline，同一則狀態再收到時不更新 store；命令回傳的目前狀態寫進 `connectionStates`；登出與移除連線清掉該 connection 的狀態。
  - `apps/desktop/src/__tests__/projectRail.test.tsx`：沒有 session 的遠端分頁、connection 為 offline 時方塊 `data-tab-dot` 為 `warning`、提示含「離線」且不含「最後資料」；connection 為 online 時沒有點；有 recovery 為 error 時仍為紅點。
  - `apps/desktop/src/__tests__/tray.test.ts`：`buildTraySnapshot` 對沒有 session、connection 為 offline 的遠端分頁回 `status: "offline"`、`hasSession: false`，`tabStatusLabel` 回「離線」；needs-reauth 時 `failureKind` 為 `needs-reauth`。
  - `cargo test -p speclink-desktop` 與 `npm test -w @speclink/desktop` 都通過。
- **範圍**：In — 上述 Rust 方法與 command、adapter、store、`remoteWorkspaceStatus`、`ProjectRail`、`buildTraySnapshot`、`tabStatusLabel`、兩個 i18n 鍵、`App.tsx` 的接線與測試。Out — Non-Goals 所列各項，以及 Server、CLI、引擎的任何改動。

## Risks / Trade-offs

- [每台 Server 每 60 秒多一個請求，主視窗隱藏時也照跑] → 請求很小，只查身分；同一條 connection 不重疊。
- [檢查時 access token 過期，會先用 refresh token 換發] → 換發失敗才轉 needs-reauth，這正是要讓使用者看到的狀態。
- [作用中分頁的事件會被套用兩次（全域監聽與既有的 session 轉交）] → 第二次套用時狀態與訊息都沒變，store 不更新；事件只在狀態轉換時發出，頻率很低。
- [檢查命令的回傳與事件的送達順序不保證] → 回傳的是檢查結束時的狀態；萬一之後的轉換事件先到、較舊的回傳後到，呈現最多錯到下一次檢查（60 秒內，或主視窗取得焦點時）。回到 online 時的資料重新整理仍只由既有轉交觸發一次。
- [回歸對照] → 不動 CLI 與引擎，golden 與 CLI 測試不受影響。
- [跨平台] → 只用計時與 HTTP，沒有路徑處理；Rust 端的等待在 `spawn_blocking` 執行緒裡，不卡住 UI。

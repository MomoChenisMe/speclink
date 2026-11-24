## Why

使用者是用 Speclink Desktop 同時開著多個專案、其中有遠端專案的開發者；情境是圖示列與系統匣的專案狀態呈現。

目前遠端專案的 Server 連不上時，圖示列要等使用者點下那個專案才會標出來。原因是沒點過的遠端分頁沒有任何程式去檢查連線：app 啟動時只檢查背景的本機分頁；只有作用中的分頁會定期問 Server；沒有 session 的遠端分頁一律畫成正常。已經點開過、現在在背景的遠端分頁也一樣，只有作用中分頁收得到連線狀態的通知。

Rust 端本來就以 Server 為單位記錄連線健康狀態，也已經會廣播狀態變化，缺的只是「沒有作用中分頁時，誰去送請求」與「前端把狀態交給沒有 session 的分頁」。這是來源討論 `rail-remote-status-and-continue-local` 的第二刀。

## What Changes

- 新增 Tauri command `remote_check_connection`：對一條 connection 送一次身分查詢（`/api/speclink/v1/whoami`，舊版 Server 改問根層 `/auth/whoami`），經既有的連線狀態判定計算成功或失敗；失敗而仍為 online 時，數秒內快速重試到成功、轉為 offline 或達到閾值次數為止；needs-reauth 時不送請求。只查身分，不讀專案資料、不建立 session、不 handshake。
- 前端定期觸發這個檢查：app 還原分頁之後、主視窗取得焦點時，以及之後每 60 秒；同一條 connection 有多個遠端分頁時只檢查一次，同一條 connection 的檢查不重疊；沒有遠端分頁時不檢查。
- 前端改為全域監聽 `remote-connection-state`，每條 connection 記一份狀態，並把事件套用到所有同 connection 的 session（不再只靠作用中分頁轉交）。
- 沒有 session 的遠端分頁以所屬 connection 的狀態呈現：offline 或 needs-reauth 時，圖示列方塊亮琥珀點；offline 的提示是新的「離線」，不提「最後資料」（這類分頁沒有資料），needs-reauth 沿用既有的需要登入文字；系統匣的專案列呈現同一個狀態。connection 回到 online 時拿掉琥珀點，不自動 handshake。分頁自己的復原狀態（連線中、錯誤）仍然優先。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `remote-resilience`：新增「背景遠端分頁的連線定期檢查」需求。既有的「離線狀態機單一真相且明確呈現」規定了狀態由 Rust 判定與分頁要呈現，但只涵蓋有 session 的分頁；這次補上沒有 session 的分頁從哪裡取得狀態、何時檢查。desktop-config「專案分頁列存於 app 本機」已寫「remote 專案離線或需重新登入時方塊角落 SHALL 為琥珀點」，tray-status-menu 已寫 remote tab 可辨識 offline 與 needs-reauth，兩者的文字不用改，所以呈現面的情境一併寫在新需求裡，不另開這兩份 delta。

## Impact

- Affected specs: `remote-resilience`（ADDED「背景遠端分頁的連線定期檢查」）
- Affected code（只影響 apps/desktop；引擎、CLI、Server、Node SDK 不動）:
  - Modified: `apps/desktop/src-tauri/src/remote.rs`（`TokenManager` 新增連線檢查方法與測試）
  - Modified: `apps/desktop/src-tauri/src/lib.rs`（新增 `remote_check_connection` command 並註冊）
  - Modified: `apps/desktop/src/adapter/connections.ts`（`checkConnection`、`onConnectionState`）
  - Modified: `apps/desktop/src/session.ts`（`remoteWorkspaceStatus` 接受 connection 狀態）
  - Modified: `apps/desktop/src/store.ts`（`connectionStates`、`checkRemoteConnections`、`startConnectionChecks`，`restoreTabs` 之後觸發）
  - Modified: `apps/desktop/src/App.tsx`（掛全域監聽、啟動定期檢查、取得焦點時檢查、把 connection 狀態交給圖示列）
  - Modified: `apps/desktop/src/components/ProjectRail.tsx`（沒有 session 的遠端分頁讀 connection 狀態）
  - Modified: `apps/desktop/src/tray.ts`（`buildTraySnapshot` 帶入 connection 狀態與 `hasSession`，`tabStatusLabel` 選用新文字）
  - Modified: `apps/desktop/src/i18n/messages.ts`（新增 `remote.recovery.offlineNoDataShort`、`tray.recovery.offlineNoData`）
  - Modified: `apps/desktop/src/__tests__/projectRail.test.tsx`、`apps/desktop/src/__tests__/tray.test.ts`
  - New: `apps/desktop/src/__tests__/connectionCheck.test.ts`
- 相容性影響：沒有 CLI 指令、`--json` 輸出、設定欄位或技能的變動。新增一個 Tauri command；既有 command 與 `remote-connection-state` 事件的形狀不變。每台有遠端分頁的 Server 每 60 秒多一個身分查詢請求。
- 三種形態：Desktop 連到的 Server 可能是官方示範 server 或內嵌宿主，兩者都依 client-protocol 提供 `/api/speclink/v1/whoami`；舊版 Server 沒有這個路徑（回 404 或 405）時改問根層的 `/auth/whoami`，與登入時查身分的做法相同。CLI 與沒有 Desktop 的內嵌宿主不適用。

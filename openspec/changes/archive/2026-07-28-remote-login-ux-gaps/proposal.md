## Why

遠端登入鏈有三個 UX 斷點，共同根因是「資料都到位、呈現層沒接」：(1) 使用者看不到自己隸屬哪些專案——web 端沒有任何一面顯示個人專案隸屬，後端卻早已為管理頁計算 memberships；(2) desktop 登入時瀏覽器要求確認裝置碼，但 desktop 全程不顯示該碼——授權回應的 user_code、verification URI、有效期限都在手上，連線互動狀態卻沒有任何變體承載它們，等待授權整段只顯示忙碌中，瀏覽器開錯或想換裝置授權時無路可走、也無法取消；(3) 授權完成後兩側都沒有銜接——desktop 停在已登入不引導開工作區，web 核准完成頁只留一行結果、不告知可返回 app。

目標使用者：使用 Remote Store 的團隊成員（含管理員）——desktop 登入 server 的使用者與在瀏覽器完成裝置核准的使用者。使用情境：對應 remote workspace 的連線與登入階段（開工作區之前的入口流程）。

## What Changes

1. **web「我的專案」**：`/account` 新增我的專案區塊，顯示使用者隸屬的專案（名稱與角色）；admin 與一般成員共用同一頁（兩種殼都經 header 帳號入口到達）。account summary API 增含 memberships（camelCase）。
2. **desktop 等待授權面**：device 授權等待期間，發起登入的介面（伺服器頁籤連線列與工作區選擇器的新增並登入區）就地顯示裝置碼與驗證網址（各附複製）、有效期限倒數與取消操作；取消即停止輪詢、不留 credential。
3. **授權後銜接**：desktop 登入成功後於連線列呈現顯眼的開啟工作區行動呼籲並取得鍵盤焦點；web 核准結果頁補「可返回 Speclink app 繼續」的收尾文案。

## Non-Goals

- 不改變 device flow 的授權語意、輪詢狀態機、token 與 credential 處理——只補呈現與取消。
- 不自動開啟工作區選擇器（會打斷多 server 登入流與 session 恢復）。
- /account 不內嵌 repo 清單與跳轉（需 admin-only API 下放且 web 端無跳轉目的地）。
- 不合併 /admin/registry（治理視角）與我的專案（個人視角）。
- 不動 PAT fallback 路徑與登出、移除流程。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `server-identity`: account summary 增含使用者自己的專案隸屬（metadata 邊界不變）。
- `server-web-console`: 帳號頁新增我的專案區塊（ADDED）。
- `server-device-auth`: 核准結果頁補返回 app 的收尾指引。
- `desktop-connections`: device 授權等待狀態呈現裝置碼、驗證網址、倒數與取消；登入成功後的開啟工作區行動呼籲。

## Impact

- Affected specs: server-identity、server-web-console、server-device-auth、desktop-connections
- Affected code:
  - Modified: crates/speclink-server/src/web.rs、crates/speclink-server/src/admin.rs、crates/speclink-server/tests/web_account.rs、apps/server-web/src/api/client.ts、apps/server-web/src/pages/AccountPage.tsx、apps/server-web/src/pages/ActivatePage.tsx、apps/server-web/src/i18n/messages.ts、apps/server-web/src/__tests__/account.test.tsx、apps/server-web/src/__tests__/activate.test.tsx、apps/server-web/src/__tests__/admin-console-shell.test.tsx、apps/server-web/src/__tests__/app.test.tsx、apps/server-web/src/__tests__/wording.test.tsx、apps/desktop/src-tauri/src/connections.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/tests/login_orchestration.rs、apps/desktop/src-tauri/tests/common/mod.rs、apps/desktop/src/store.ts、apps/desktop/src/adapter/connections.ts、apps/desktop/src/components/ServersPanel.tsx、apps/desktop/src/components/WorkspaceChooser.tsx、apps/desktop/src/App.tsx、apps/desktop/src/i18n/messages.ts、apps/desktop/src/__tests__/serversPanel.test.tsx、apps/desktop/src/__tests__/workspaceChooser.test.tsx、apps/desktop/src/__tests__/store.test.ts、apps/desktop/src/__tests__/remoteOpen.test.ts、apps/desktop/src/__tests__/remoteResilience.test.tsx
  - New: apps/desktop/src/components/connectionLogin.tsx（等待授權面與 PAT 輸入的共用元件——伺服器頁籤連線列與工作區選擇器共用）
  - Removed: （無）

影響的 crate 或 app：speclink-server（account summary API）、apps/server-web（帳號頁、核准頁）、apps/desktop（Tauri 編排與伺服器頁籤 UI）。

相容性影響：CLI 零變更。account summary 的 --json 無涉（browser API 新增欄位、既有欄位不動，屬向後相容擴充）。desktop 的 device login 編排由單一阻塞呼叫改為分段（內部 IPC 形狀變更，無對外契約）；人眼可見變化為新增的等待授權面與行動呼籲，均為刻意的 UX 補強。

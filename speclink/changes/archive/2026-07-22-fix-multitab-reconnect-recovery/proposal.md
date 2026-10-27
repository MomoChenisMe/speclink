## Why

目標使用者是透過 Speclink Desktop 同時處理多個 Remote workspace 的開發者、PO 與 PM，使用情境涵蓋 apply、審查與驗收期間的伺服器重啟及分頁切換。現在同來源多分頁在伺服器恢復後可能把正常的自動重連誤判成 refresh token 重放而撤銷整個 credential family；切換到需重新驗證的分頁時，也可能暫時保留上一個 workspace 的內容，讓使用者在錯誤的上下文中閱讀或操作資料。

## What Changes

- 修改 Remote resilience 契約：同來源多個 session 同時恢復時，所有呼叫者必須共用一次成功的認證恢復結果，正常伺服器重啟不得因本機併發而進入 `needs-reauth`。
- 修改 Workspace session 契約：畫面中的 workspace 內容必須屬於目前 active session；切換後若載入失敗，只能顯示該 session 自己最後成功的快照，或安全的載入／恢復／空白狀態。
- 補上可重現 refresh credential 併發輪替與跨分頁非同步載入競態的決定性回歸測試。
- 保留明確撤銷、真正重放或 credential 無效時進入重新驗證的既有行為。

## Non-Goals

- 不放寬伺服器端 refresh token 一次性輪替與重放撤銷規則，也不增加寬限時間。
- 不變更 Keychain credential schema、access token 僅存記憶體的原則、Remote Protocol 或 Server API。
- 不新增 Remote workspace 能力、不重設 Desktop 分頁介面，也不在此 change 直接完成 `phase3-e2e` 的剩餘人工驗收。
- 不變更 CLI、設定欄位、技能或 CLAUDE.md／AGENTS.md 注入區塊。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `remote-resilience`: 明確規範同來源多 session 併發恢復只執行一次 credential 輪替，並讓正常重啟自動恢復而不誤入重新驗證。
- `workspace-session`: 明確規範 active session 的可見內容所有權、每個 session 的最後成功快照，以及過期非同步結果不得覆寫目前分頁。

## Impact

- Desktop 認證與 Remote runtime：`apps/desktop/src-tauri/src/remote.rs`、相關 Desktop Rust 測試。
- Desktop session state 與呈現：`apps/desktop/src/store.ts`、必要的 `apps/desktop/src/App.tsx` 邊界，以及相關 React／store 測試。
- `speclink-core`、`speclink-cli`：無影響。
- API、相依套件、資料庫、設定與部署：無變更。
- 相容性影響：CLI 人眼輸出與 `--json` 均不變，既有回歸對照與使用者遷移方式不變。

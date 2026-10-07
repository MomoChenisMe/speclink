## Problem

官方 `speclink-server` 的專案成員有 `editor` 與 `reader` 兩種角色，reader 的定位是「檢視者」，但 server 沒有一致地擋下 reader 的寫入：

- 只有 12 個寫入端點檢查角色（change depends、刪除變更、任務搬移、審查與驗證的蓋章與放棄、認領、刪除討論、policy 寫入、看板排序、匯入）。
- 其餘 17 個寫入端點任何成員都能呼叫：建立變更、寫 artifact、勾選與取消勾選任務、審查與驗證新增輪、開工標記與移除、封存、建立討論、討論的 link／seal／context／新增輪／結論／封存／轉為變更。
- 用 reader 帳號登入的 CLI（`speclink link` + `speclink auth login`）照樣能建立變更、改規格、封存，等於 reader 沒有意義。
- 桌面 app 的遠端看板對 reader 也沒有全面停用寫入：勾任務、全勾、封存、轉為變更與封存討論，capability 寫死為可用；reader 按下去不會被擋。

另外，server 對內容格式錯誤回 500 `internal`：寫入的 tasks.md 沒有 `- [ ]`、或審查輪缺 `**Scope**:` 行時，使用者看到的是「伺服器內部錯誤」，而不是「你送的內容不合格式」。

使用者是透過 AI 代理跑 SDD、以 Remote Store 協作的團隊（PO／PM／RD），以及管理 server 的管理員；情境是 Remote 模式下的 propose、apply、品質關卡、archive 與 discuss 各站。

## Root Cause

- 角色檢查散落在個別處理函式（逐一寫 `if !binding.editor`），每加一個寫入端點就要記得手動補，漏掉的端點預設開放（fail-open）。
- 引擎把內容格式檢查失敗（`validate_artifact_content` 與品質關卡的輪內容解析）歸類為通用的 `error` 錯誤碼，server 依錯誤碼對照表把 `error` 一律轉成 500。

## Proposed Solution

1. **server：寫入一律要求 editor，檢查集中在一個位置**（speclink-server）
   - 在專案範圍的路由加一道路由層檢查：方法不是 GET／HEAD 的請求，呼叫者必須是 editor，否則回 403 `permission_denied`，不執行處理函式、不寫入任何東西。
   - 唯讀但用 POST 的 `POST /context`（文件快照）排除在外，在檢查之後才註冊，reader 照常可用。
   - 新增的寫入端點自動受保護（預設擋下，fail-closed）。
   - 移除 12 個處理函式裡分散的角色檢查。
2. **引擎：內容格式錯誤歸類為 `invalid_argv`**（speclink-core）
   - 新增一個內容格式錯誤的型別標記，`validate_artifact_content` 與輪內容解析改用它；命令層分類時歸為 `invalid_argv`。
   - server 依既有對照表回 400 `invalid_argument`；錯誤訊息文字不變。
3. **CLI：403 的訊息帶出 server 的原因**（speclink-remote）
   - 遠端模式收到 403 時，CLI 目前一律印「access denied — your account has no access to this project; ask a project admin」。reader 寫入被擋時，這句話會誤導成「沒有權限看這個專案」。
   - 改成 `access denied — <server 訊息>; ask a project admin`：非成員看到「actor is not a member of project '<代號>'」，reader 寫入看到角色原因。
4. **桌面 app：遠端看板依角色停用全部寫入**（apps/desktop）
   - 遠端 capability 的勾任務、全勾、封存、轉為變更、封存討論改為依 handshake 的角色決定，與刪除變更、認領一致。
   - 停用說明依原因分開：reader 顯示「你的角色為檢視者，只能查看」；離線仍顯示原本的唯讀說明。
5. **文件**：docs/verb-contract 兩版的端點總表與角色說明改成「所有寫入都要 editor」；docs/sdk-node 兩版補充內容格式錯誤的錯誤碼。

## Non-Goals

- 不做更細的權限（repo 級角色、逐動作權限、管理員以外的新角色）。2026-07-20 的設計已定為只有 editor／reader 兩種，本變更只補齊兩種角色的執行面。
- 不設 reader 可寫的例外（例如允許 reader 寫討論）：桌面已把 reader 的全部寫入停用，例外會讓 CLI 與桌面不一致。
- 不新增 handshake 的 capability 欄位：沿用既有作法，以 `deleteChange` 代表「這個成員可寫變更」。
- 除了遠端 403 的訊息之外，不改 CLI 的人眼輸出與 exit code；引擎的錯誤訊息文字不變。
- 不處理本地（Local Repo）模式：本地沒有角色。

## Success Criteria

- reader 呼叫任何一個寫入端點，都回 403 `permission_denied`，專案資料零改動；editor 的行為不變。
- reader 呼叫 `POST /context` 與所有 GET 端點照常成功。
- 寫入格式錯誤的 tasks.md 或缺 `**Scope**:` 的審查輪，server 回 400 `invalid_argument`、訊息文字與 CLI 相同；Node SDK 的錯誤碼為 `invalid_argv`。
- 桌面遠端看板以 reader 開啟時，勾任務、全勾、封存、轉為變更、封存討論、刪除、搬移、排序、認領、前置編輯全部停用，說明文字是角色原因；離線時的說明維持原樣。

### 相容性影響

- **reader 不能再寫入**：原本以 reader 身分透過 CLI 或 HTTP 寫入的使用方式會改收 403。需要寫入的人，請管理員在 `/admin/users` 把角色改成 editor。
- **遠端 403 訊息**：CLI 在遠端模式收到 403 時，`access denied — ` 後面改接 server 的原因，不再是固定句子。開頭仍是 `access denied`。
- **錯誤碼**：內容格式錯誤從 HTTP 500 `internal` 改成 400 `invalid_argument`；Node SDK 的錯誤碼從 `error` 改成 `invalid_argv`。CLI 的人眼輸出、`--json` 與 exit code 不變，不影響 golden 回歸對照。
- 桌面 app 接舊版 server 時，reader 的寫入按鈕一樣停用（判斷來自 handshake 角色，不依賴新 server）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `server-verb-api`: 新增「寫入端點一律要求 editor 角色」的需求，涵蓋全部專案範圍寫入端點與 `POST /context` 的例外。
- `remote-workspace-data`: reader 的寫入面停用範圍擴及勾任務、全勾、封存、轉為變更、封存討論，並區分角色與離線兩種停用說明。
- `command-runtime`: 錯誤碼註冊表載明「內容格式檢查失敗」歸類為 `invalid_argv`。


## Impact

- Affected specs: server-verb-api, remote-workspace-data, command-runtime
- 掃描到但不需修改的正式規格：server-policy-write（policy 寫入對 reader 回 403，維持不變）、client-protocol（invalid_argv 對應 400 invalid_argument 的對照已存在）
- Affected code:
  - Modified: crates/host/speclink-server/src/app.rs
  - Modified: crates/host/speclink-server/src/api/routes.rs
  - Modified: crates/host/speclink-server/src/identity/auth.rs
  - Modified: crates/engine/speclink-core/src/command/mod.rs
  - Modified: crates/engine/speclink-core/src/lifecycle/newcmd.rs
  - Modified: crates/engine/speclink-core/src/quality/station.rs
  - Modified: crates/protocol/speclink-remote/src/lib.rs
  - Modified: crates/protocol/speclink-remote/tests/it/typed_client.rs
  - Modified: apps/desktop/src-tauri/src/remote.rs
  - Modified: apps/desktop/src/App.tsx
  - Modified: apps/desktop/src/i18n/messages.ts
  - Modified: docs/verb-contract.zh-TW.md
  - Modified: docs/verb-contract.md
  - Modified: docs/sdk-node.zh-TW.md
  - Modified: docs/sdk-node.md
  - New: crates/host/speclink-server/tests/it/api/write_guard.rs
  - Modified: crates/host/speclink-server/tests/it/api/mod.rs

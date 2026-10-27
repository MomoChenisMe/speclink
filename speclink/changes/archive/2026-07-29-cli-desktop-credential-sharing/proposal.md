## Why

desktop 與 CLI 是同一顆引擎的兩個殼，卻各持一套互不知情的憑證儲存：desktop 走 OS Keychain（refresh credential＋PAT 備援），CLI 只認 SPECLINK_TOKEN 環境變數與使用者設定目錄的憑證檔。結果是 desktop 完成裝置授權登入後，remote 模式下的每個 speclink 動詞（所有技能的前置）仍報未登入，使用者被迫手動建 PAT 再登入一次。根因是驗證編排放錯層——refresh 換發與 Keychain 存取留在 desktop app 層，未下沉到共用的 speclink-remote。

目標使用者：透過 AI 代理跑 SDD 的開發者。兩種情境都要顧：desktop＋CLI 並用者（登入一次、兩邊可用），與不裝 desktop 的純 CLI 使用者（獲得免手建 PAT 的裝置授權登入，既有 PAT／CI 路徑零退化）。

## What Changes

- **憑證儲存與換發編排下沉至 speclink-remote**：CredentialStore（keyring 生產實作＋in-memory 測試實作）與 refresh 換發編排從 desktop app 層移入 speclink-remote，desktop 改為呼叫共用層。Keychain 條目的 service 與 account 鍵維持不變——既有 desktop 登入無需遷移。
- **同機同 origin 共用一個 credential family**：desktop 與 CLI 讀寫同一個 Keychain refresh 條目；換發（讀 refresh → 呼叫 server 換發端點 → 回寫新 refresh）全程持使用者設定目錄的檔案鎖序列化，防止兩行程併發換發觸發 server 端 reuse 偵測的整族撤銷。
- **CLI 憑證解析階梯**：SPECLINK_TOKEN → Keychain refresh 換發 → Keychain PAT → 憑證檔 PAT，任一層不可用（含平台無 keyring）即靜默下探。既有兩層的相對順序不變。
- **speclink auth login 雙軌**：互動 TTY 下無旗標即走裝置授權（能開瀏覽器則開啟核准頁，否則印 verification URL 與裝置碼供他機核准），成功後 refresh 存入 Keychain。新增 --pat 旗標保留互動貼 PAT；--token-stdin 行為與儲存位置（憑證檔）完全不變。非互動且無旗標時以非 0 exit code 報錯並指引 --token-stdin。
- **speclink auth status 顯示憑證來源層**：人眼輸出與 --json 各增列憑證來自階梯哪一層；既有欄位不動。
- **speclink auth logout 新增**：持有 refresh 時呼叫 server 撤銷該 credential family，並清除該 origin 的本機憑證（Keychain 條目與憑證檔條目）；未登入時非 0 exit code。共用一族的對稱後果：任一端登出即全機登出。

相容性影響：

- 人眼輸出——speclink auth login 互動預設從「貼 PAT」變為裝置授權（**BREAKING**，互動情境限定）；要舊行為加 --pat。CI／腳本用的 --token-stdin 不受影響。
- --json——auth status 新增來源層欄位（加欄位、不改既有欄位，camelCase）。
- 既有 CLI 測試與 golden 同批更新，變更於本提案記載。

## Non-Goals

- headless／無 keyring 環境的裝置授權憑證持久化：refresh 不落明文檔（rotation 回寫＋reuse 偵測使檔案副本成為地雷），該環境維持 PAT／SPECLINK_TOKEN 現狀。
- PAT 改寫入 Keychain：憑證檔路徑維持現狀，未來再議。
- server 端變更：裝置授權、換發、撤銷端點皆已存在，本變更只是讓 CLI 成為新的消費者。
- desktop UI 與登入流程變更：desktop 使用者可見行為不變。
- 每客戶端一族（desktop 與 CLI 各自登入）：無法消除重複登入，已於討論中否決。
- macOS Keychain 對第二個 binary 的首次存取系統提示無法繞過，僅於文件與錯誤訊息說明，不做額外處理。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `remote-auth`: 登入從單軌 PAT 擴為雙軌（裝置授權預設＋PAT 旗標）；憑證解析從兩層擴為四層階梯（新增 Keychain refresh 換發與 Keychain PAT）；新增跨前端共用 credential family 與換發序列化要求；新增 auth logout；auth status 增列憑證來源層。

## Impact

- Affected specs: `remote-auth`（修改）
- Affected code:
  - New: `crates/speclink-remote/src/credentials.rs`（CredentialStore 下沉）、`crates/speclink-remote/src/refresh.rs`（換發編排＋檔案鎖）
  - Modified: `crates/speclink-remote/src/auth.rs`、`crates/speclink-remote/src/lib.rs`、`crates/speclink-cli/src/main.rs`、`crates/speclink-cli/src/remote_commands.rs`、`apps/desktop/src-tauri/src/connections.rs`、`apps/desktop/src-tauri/src/remote.rs`、`apps/desktop/src-tauri/Cargo.toml`
  - Removed: `apps/desktop/src-tauri/src/credentials.rs`（內容下沉至 speclink-remote 後刪除）
- 影響的 crate／app：speclink-remote、speclink-cli、apps/desktop（src-tauri）；speclink-server 不動。
- 相依：speclink-remote 新增 keyring 相依（自 desktop 移入）與檔案鎖相依。

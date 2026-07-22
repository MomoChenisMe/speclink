## Problem

目標使用者是透過 Speclink Desktop 連接 Remote Store、並由 AI 代理執行 SDD 工作流的開發者、PO 與 PM。Desktop 啟動裝置授權時雖已在瀏覽器 URL 帶入裝置碼，但未登入使用者完成登入後會遺失啟用上下文，且啟用頁未預填裝置碼，導致遠端連線流程無法完成；此問題目前也阻擋 Phase 3 真實視窗端對端驗收。

## Root Cause

Server 的裝置啟用入口未接收 Desktop 已附帶的 `user_code` 查詢參數；未登入時固定前往一般登入頁，而一般登入成功後固定前往帳號頁。即使瀏覽器已有登入 session，啟用頁也只呈現空白裝置碼欄位，因此整條瀏覽器流程無處保留或恢復 Desktop 建立的短碼。

## Proposed Solution

- 裝置啟用頁在收到有效格式的 URL 裝置碼時預填輸入欄位，但仍要求使用者完成既有的明確確認與核准步驟。
- 未登入使用者由裝置啟用頁前往登入時，只保留格式合格的裝置碼；成功登入後由 Server 固定返回同一啟用流程。
- 不安全或格式不合的值不予傳遞或反映，登入成功後維持既有的帳號頁回退行為。
- 直接登入、直接開啟無裝置碼的啟用頁、失敗訊息一致性、裝置輪詢及憑證保存語意維持不變。
- 以伺服器整合測試覆蓋從裝置啟用 URL、登入、預填、確認、核准到輪詢成功的完整瀏覽器形狀流程。

## Non-Goals

- 不建立通用或可任意指定目的地的登入後轉址機制。
- 不自動核准裝置、不移除明確確認頁，也不因 URL 中的裝置碼洩漏其存在、過期或使用狀態。
- 不改造 Desktop 介面、不另行顯示或複製裝置碼，也不變更 Keychain（作業系統鑰匙圈）儲存流程。
- 不變更 PAT（個人存取權杖）、CLI 認證、裝置輪詢協定、資料庫結構或 token（權杖）格式。
- 不修改 `phase3-e2e` 變更的產品程式碼範圍；本修正以獨立 Bug Fix 交付。

## Success Criteria

- 瀏覽器沒有 Server session 時，Desktop 開啟的裝置啟用 URL 可經登入返回同一啟用頁，且裝置碼已預填。
- 瀏覽器已有 Server session 時，裝置啟用頁直接預填裝置碼。
- 預填不會自動查驗或核准，使用者仍須經過下一步及明確核准／拒絕確認。
- 缺少、格式不合或遭竄改的裝置碼不出現在登入頁、啟用頁或返回 Location；直接登入仍前往帳號頁。
- 完整 HTTP 裝置鏈可在核准前維持 pending，核准後輪詢取得 approved 與憑證；既有 Server 及 workspace 回歸測試通過。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `server-device-auth`: 裝置啟用入口須安全保留登入往返中的裝置碼、預填 URL 裝置碼，並維持明確確認及無資訊洩漏的既有授權語意。
- `desktop-connections`: Desktop 啟動的瀏覽器裝置登入流程須在未登入情境下可經登入往返後繼續完成授權。

## Impact

- Affected specs：`server-device-auth`、`desktop-connections`。
- Affected code：
  - Modified：`crates/speclink-server/src/web.rs`、`crates/speclink-server/tests/web_activate.rs`、`crates/speclink-server/tests/web_account.rs`、`crates/speclink-server/tests/device_e2e.rs`。
  - New：無。
  - Removed：無。
- 影響 `speclink-server` crate 的 Web 登入與裝置啟用行為；`speclink-core`、`speclink-cli` 與其他 crate 不受影響。
- 沒有新依賴、設定欄位、CLI 子指令、旗標、stdin 或 exit code 變更，也不影響 claude／codex 技能或注入區塊。
- 相容性影響：既有 CLI 人眼輸出與 `--json` shape 均不變，不破壞 Spectra 回歸對照；直接登入成功仍前往帳號頁，既有使用者不需遷移。瀏覽器中唯一可見差異是從 Desktop 裝置啟用流程登入後返回啟用頁，且裝置碼已預填。

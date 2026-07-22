## 1. Red：重現裝置啟用登入斷鏈

- [x] 1.1 先在 `crates/speclink-server/tests/web_activate.rs` 為「核准頁 session 保護且明確確認」及「預填輸入但保留明確確認」撰寫失敗整合測試：驗證未登入查詢會保留格式合格短碼、已登入頁面只預填而不查驗或核准、無參數保持空白、不合格式值不反映；執行 `cargo test -p speclink-server --test web_activate`，確認新增案例因現有 GET 忽略 `user_code` 而以正確原因失敗。 <!-- speclink-task:tsk_01KY40JFG0E3HR1NWC4F7SZ60K -->
- [x] 1.2 先在 `crates/speclink-server/tests/web_account.rs` 為「使用專用 user_code 傳遞啟用上下文」與「在每個 Web 邊界重新驗證短碼格式」撰寫失敗整合測試：驗證登入頁隱藏欄位、有效短碼登入後固定返回 `/activate?user_code=...`、失敗登入保留短碼且未知 email／錯誤密碼 byte-identical、直接登入及不合格式值回退 `/account`；執行 `cargo test -p speclink-server --test web_account`，確認新增案例因現有登入流程固定前往 `/account` 而以正確原因失敗。 <!-- speclink-task:tsk_01KY40JFG0RTDPKJBJP840FHD3 -->
- [x] 1.3 先在 `crates/speclink-server/tests/device_e2e.rs` 為「device login 預設與 PAT fallback」及「以瀏覽器形狀整合測試驅動實作」新增失敗回歸：以真實 HTTP、cookie 與 Location 重現 initialize → 未登入啟用 URL → 登入 → 預填 → 下一步 → 明確核准 → poll approved；執行 `cargo test -p speclink-server --test device_e2e`，確認鏈在登入後未返回啟用頁處失敗。 <!-- speclink-task:tsk_01KY40JFG0V5BREBHFJTGR4M1C -->

## 2. Green：最小化修復登入往返與預填

- [x] 2.1 在 `crates/speclink-server/src/web.rs` 實作專用 `user_code` query／form 傳遞、每個外部邊界的現行 `XXXX-XXXX` ASCII 格式驗證、Server 固定建構的啟用返回 Location，以及 HTML-escaped 預填；不得接受任意 URL、不得在 GET 查詢或改變裝置狀態，並以 `cargo test -p speclink-server --test web_activate --test web_account --test device_e2e` 證明 1.1–1.3 全部由紅轉綠。 <!-- speclink-task:tsk_01KY40JFG0DHPKMMKA3NTXRRP3 -->
- [x] 2.2 在 `crates/speclink-server/src/web.rs` 保持既有直接登入、空白啟用頁、POST 同源保護、明確核准／拒絕及統一無效回應不變；執行 `cargo test -p speclink-server --test web_account --test web_activate`，確認新舊案例全數通過且無未要求的 Web 行為變更。 <!-- speclink-task:tsk_01KY40JFG0T2NFQSBQ4CX37TFA -->

## 3. Refactor 與 sharp-edges audit

- [x] 3.1 僅在 `crates/speclink-server/src/web.rs` 合併重複的短碼驗證與固定 URL 組裝、校正命名與註解，不新增通用轉址抽象；每個小步後執行 `cargo test -p speclink-server --test web_activate --test web_account --test device_e2e`，確認行為持續為綠。 <!-- speclink-task:tsk_01KY40JFG073WVD5S50A13VQ8B -->
- [x] 3.2 依 `speclink instructions --skill audit` 的 discipline 模式檢查 `crates/speclink-server/src/web.rs` 與三份測試的參數注入、危險預設、靜默安全失敗與字串型安全陷阱；修正所有 Critical／High 發現，並以惡意或不合格式 `user_code` 不進入 HTML／Location、有效登入仍安全回退的測試結果作為完成證據。 <!-- speclink-task:tsk_01KY40JFG0P32HMYPGRSQPPKQF -->

## 4. 完整回歸與變更驗證

- [x] 4.1 執行 `cargo test -p speclink-server`，確認 Server 所有認證、session、裝置狀態機、PAT 與 Web 整合測試全數通過，且沒有新依賴、設定、資料庫、protocol 或 Desktop 產品碼變更。 <!-- speclink-task:tsk_01KY40JFG08F288SY8Z7RKT5V3 -->
- [x] 4.2 執行 `cargo test --workspace` 與 `cargo build --release`，再執行 `target/debug/speclink analyze device-activation-query-prefill --json` 及 `target/debug/speclink validate device-activation-query-prefill`；確認跨 crate 回歸、release build、規格對應與 artifact 結構皆無阻擋問題，CLI 人眼輸出及 `--json` parity 未被修改。 <!-- speclink-task:tsk_01KY40JFG06ANWGC8E3CTMKS85 -->

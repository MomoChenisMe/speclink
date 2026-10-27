## Context

Desktop 的裝置登入編排已將 Server 回傳的 verification URI 加上 `user_code` 查詢參數後交給系統瀏覽器。Server 目前的 `GET /activate` 未讀取該參數：未登入時固定導向 `/login`，登入成功後固定前往 `/account`；已登入時則顯示空白裝置碼欄位。因此使用者既失去原啟用上下文，也無法從 Desktop 畫面取回短碼，真實裝置登入無法走完。

此修正只跨越 `speclink-server` crate 內的 server-rendered Web 登入與裝置啟用入口。裝置狀態機、protocol DTO、Desktop 編排、OS Keychain、儲存層、`speclink-core` 與 `speclink-cli` 都維持既有邊界。外部查詢參數及表單資料屬系統邊界，必須驗證；無需新增依賴、序列化格式、設定或資料庫 migration。

## Goals / Non-Goals

**Goals:**

- 讓從 Desktop 開啟的 `/activate?user_code=XXXX-XXXX` 在未登入時經過登入後仍能回到同一裝置啟用流程。
- 讓已登入的啟用頁預填格式合格的裝置碼，同時保留既有的「下一步 → 明確核准／拒絕」兩階段操作。
- 不在查詢參數處查詢裝置碼狀態，使未知、已用與逾期短碼仍只在提交後得到同一無效回應。
- 以 TDD（測試驅動開發）的紅、綠、重構順序固定正常流程、安全回退與既有相容性。

**Non-Goals:**

- 不接受任意登入後返回 URL，不建立通用轉址框架。
- 不自動提交或核准裝置碼，不縮短既有明確確認步驟。
- 不修改 Desktop UI、裝置輪詢、token／credential、Keychain、PAT、CLI、資料庫或設定。
- 不改動 `phase3-e2e` 的產品程式碼範圍，也不藉此重構其他 Web 表單。

## Decisions

### 使用專用 user_code 傳遞啟用上下文

`GET /activate` 讀取可選的 `user_code` 查詢參數。未登入且參數符合現行八字元、中央連字號及排除易混淆字元的短碼格式時，Server 導向 `/login?user_code=...`；無參數或格式不合時仍導向 `/login`。登入表單以同名隱藏欄位傳遞短碼，登入成功後只由 Server 重建 `/activate?user_code=...`。

選擇專用欄位而非 `return_to` URL，是因為本變更只有一個返回目的地；它從資料形狀上排除外部、scheme-relative（雙斜線開頭）及其他本站路徑的開放轉址。替代方案是允許並驗證同源 `return_to`，但會擴大攻擊面與測試矩陣，且沒有目前需求。

### 在每個 Web 邊界重新驗證短碼格式

Server 對啟用頁查詢、登入頁查詢及登入 POST 表單中的 `user_code` 各自套用同一個純格式驗證規則，只接受現行產生器可產出的 `XXXX-XXXX` ASCII 短碼。格式不合即視為未提供，不反映到 HTML 或 Location，也不影響登入本身。

選擇每個外部邊界重新驗證，是因為隱藏欄位可被瀏覽器端修改；不能信任先前頁面的驗證結果。替代方案是只做 HTML escaping（跳脫）後照樣傳遞任何字串，雖可防止標記注入，仍會把不必要的攻擊者輸入帶入轉址與頁面。

### 預填輸入但保留明確確認

已登入使用者造訪帶有格式合格短碼的啟用頁時，Server 將 HTML-escaped（HTML 跳脫）的短碼放入裝置碼 input 的 `value`；未提供或格式不合時顯示既有空白欄位。GET 不讀取裝置授權狀態、不改變 pending 狀態，也不直接顯示核准／拒絕按鈕。使用者仍須 POST「下一步」，通過既有 pending 檢查後才進入明確確認頁。

替代方案是 GET 時直接查驗短碼並跳到確認頁，但那會把可觀察狀態移到 URL 存取階段、弱化明確操作邊界，並增加未知／已用／逾期狀態洩漏風險，因此不採用。

### 以瀏覽器形狀整合測試驅動實作

先在 `web_activate` 與 `web_account` 整合測試加入會失敗的案例，固定查詢傳遞、登入失敗保留、成功返回、預填及不合格式回退；再以 `device_e2e` 從初始化、未登入啟用、登入、返回、下一步、核准到輪詢 approved 建立完整 HTTP 鏈。實作只修改 `speclink-server` Web adapter，完成後執行該 crate 測試及 workspace 回歸。

替代方案是只增加單元測試或依賴真實 GUI 手動驗收；前者無法證明 cookie、Location 與表單往返，後者速度慢且不適合作為永久回歸守門。這項變更不觸及規格儲存或流程抽象，維持 storage（儲存）解耦架構不變。

## Implementation Contract

**可觀察行為**

- 未登入的 `GET /activate?user_code=ABCD-EFGH` 導向帶同一格式合格短碼的登入頁；登入成功後帶 session cookie 導回 `/activate?user_code=ABCD-EFGH`。
- 登入失敗仍回相同的「帳號或密碼不正確」回應語意，且保留格式合格短碼以供再次提交；未知 email 與錯誤密碼在相同短碼輸入下維持 byte-identical（位元組完全一致）。
- 已登入的啟用頁把格式合格短碼預填於裝置碼欄位；頁面仍只提供「下一步」，不在 GET 時核准、拒絕或查驗該碼是否 pending。
- 直接 `GET /login` 的成功結果仍為 `/account`；直接 `GET /activate` 仍顯示空白輸入；既有 POST 確認、核准、拒絕、同源保護及統一無效回應不變。
- 查詢或表單中的不合格式短碼不得出現在 HTML 或 Location；使用者完成有效帳密登入時回到 `/account`。

**介面與資料形狀**

- `GET /activate` 接受可選 query 欄位 `user_code: String`。
- `GET /login` 接受可選 query 欄位 `user_code: String`。
- `POST /login` 的 URL-encoded form 在既有 `email`、`password` 外接受預設為空的 `user_code: String`，使既有呼叫者相容。
- 只有符合現行 `XXXX-XXXX` 字元集合的短碼可跨頁傳遞；返回目的地由 Server 固定建構，外部輸入不包含 URL 或路徑。
- 所有回應仍是既有 server-rendered HTML、redirect 與 session cookie；無 JSON、CLI、IPC、檔案或設定格式變更。

**失敗模式**

- 不合格式或缺少短碼時靜默回退既有登入／帳號頁流程，不新增可區分的錯誤。
- 未知、已用或逾期但格式合格的短碼可被預填；只有使用者提交後才得到既有相同的無效回應。
- 認證、session 建立、identity storage（身分儲存）或裝置決策錯誤沿用現有狀態碼與內部錯誤處理。
- 所有變更型 POST 繼續執行既有同源檢查。

**驗收條件**

- `web_activate` 覆蓋未登入查詢傳遞、已登入預填、無參數空白、不合格式不反映、提交後才明確確認。
- `web_account` 覆蓋直接登入維持 `/account`、有效短碼登入成功返回、登入失敗保留短碼且錯誤語意一致、不合格式回退。
- `device_e2e` 以真實 HTTP 與 cookie 完成 Desktop 會產生的完整瀏覽器形狀裝置授權鏈，最後輪詢取得 approved。
- `cargo test -p speclink-server` 與 `cargo test --workspace` 通過，Speclink analyze 與 validate 無阻擋問題。

**範圍界線**

- 範圍內只有 `speclink-server` Web adapter 及相關整合測試。
- Desktop 現有 verification URL 組裝是契約前提但不需修改；任何 Desktop UI／Keychain、protocol／remote、core／cli 或資料模型改動都在範圍外。

## Risks / Trade-offs

- [Risk] 登入失敗回應因短碼不同而不再跨所有請求 byte-identical → Mitigation：安全契約限定同一短碼下未知 email 與錯誤密碼完全一致；測試同時確認不回填 email 或密碼。
- [Risk] 查詢或隱藏欄位被竄改，造成反射型 HTML 或 Location 注入 → Mitigation：每個 Web 邊界先做嚴格 ASCII 短碼格式驗證，輸出 HTML 時仍執行 escaping，Location 只由固定路徑與合格字元建構。
- [Risk] 登入與啟用既有流程回歸 → Mitigation：先固定直接登入、空白啟用、同源檢查、統一無效回應及完整裝置鏈，再做最小實作並執行 crate 與 workspace 回歸。
- [Risk] 瀏覽器對 redirect、cookie 或 URL encoding 的處理差異 → Mitigation：短碼只含 URL-safe ASCII 字元，整合測試以 Location 與 cookie 往返驗證；不使用平台路徑、檔案系統或 OS 特有 API，Windows／macOS／Linux 行為一致。

## Migration Plan

無資料或設定 migration。部署新版 Server 即可讓既有 Desktop 流程生效；舊 Desktop 已送出相同的 `user_code` 參數。回滾只需還原 Server Web 變更，不影響已存在的 session、裝置授權記錄或 credential。

## Open Questions

無。

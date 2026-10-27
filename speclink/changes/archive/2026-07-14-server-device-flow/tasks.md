## 1. protocol DTO 與 identity schema

- [x] 1.1 【紅→綠】speclink-protocol 新增 device 模組 DTO：發起回應（deviceCode、userCode、verificationUri、expiresIn、interval）、輪詢請求/回應（status：pending、slow_down、approved、expired、denied 與核准時的 accessToken、refreshToken、expiresIn）、refresh 請求/回應、revoke 請求。納入既有 JSON Schema 匯出與序列化往返測試（camelCase 驗證）。驗收：cargo test -p speclink-protocol 全綠。 <!-- speclink-task:tsk_01KXFHM1AJM8G1VQHQVS69KZJP -->
- [x] 1.2 【紅→綠】identity 資料庫 schema version 1→2：新增 device 授權請求、access token、refresh credential 與 credential family 表；migrate 升級測試——version 1 資料庫（含既有 user 與 PAT）升級後資料完整、既有 PAT 照常通行、較新版本仍拒開。涵蓋「identity schema 演進守門」情境。 <!-- speclink-task:tsk_01KXFHM1AJ1KM0VRNMW5FYW08W -->

## 2. 發起與輪詢狀態機

- [x] 2.1 【紅】針對「device 授權發起與輪詢狀態機」寫測試：發起回兩碼與 URI/到期/間隔且兩碼僅 hash 落庫；核准前輪詢 pending；低於間隔輪詢 slow_down 且不影響請求有效性；逾期輪詢 expired；拒絕後輪詢 denied；未知 device code 回 not_found wire error。驗收：測試存在且此時失敗。 <!-- speclink-task:tsk_01KXFHM1AJSQTAW864XJNNSHMD -->
- [x] 2.2 【綠】實作發起與輪詢端點（crates/speclink-server/src/device.rs）：user code 以避開易混淆字元的字母表產生、預設 15 分鐘到期；輪詢間隔檢查以最近輪詢時戳實作。2.1 全綠。 <!-- speclink-task:tsk_01KXFHM1AJ2P1YMFDYA82QZ4TR -->

## 3. 核准頁

- [x] 3.1 【紅→綠】/activate 核准頁：未登入導向 /login 且請求維持未核准；登入後輸入 user code 呈現確認步驟、核准或拒絕記錄操作者身分；未知/已用/逾期 user code 回同一無效回應；POST 沿用同源驗證。涵蓋「核准頁 session 保護且明確確認」全部情境。 <!-- speclink-task:tsk_01KXFHM1AJNGPARKEBEBJDGEBZ -->

## 4. token 核發、bearer 併入與 rotation

- [x] 4.1 【紅→綠】核准後輪詢核發 access token（spk_at_ prefix、1 小時效期、hash 落庫、綁核准者）與 refresh credential（spk_rt_ prefix、一次性、同 family）。auth.rs bearer 前置依 prefix 分流 PAT 與 access token，檢查清單共用（hash、撤銷、到期、user active、membership、無快取）。測試涵蓋「access token 短效且併入 bearer 前置」兩情境與停權 user 使 device 憑證即時失效。 <!-- speclink-task:tsk_01KXFHM1AJTBY3SREBENP63DQ7 -->
- [x] 4.2 【紅→綠】refresh 端點與 family 撤銷：換發後舊 refresh 立即失效；舊值重用撤銷整個 family（含新 access token）且該請求回 401；revoke 端點以 refresh credential 撤銷自身 family。測試涵蓋「rotation 舊值失效」情境。 <!-- speclink-task:tsk_01KXFHM1AJ7A0X89290ACJFMMT -->
- [x] 4.3 【紅→綠】帳號頁 sessions 清單納入 device credential families（建立時間、最近 refresh、核准來源）並支援逐一撤銷，撤銷即時生效且不影響其他 family 與 PAT。涵蓋「帳號頁撤銷 device session」情境。 <!-- speclink-task:tsk_01KXFHM1AJXBHNG8DNTRCD917C -->

## 5. 端到端與回歸

- [x] 5.1 【紅→綠】e2e：對真 server（SQLite store＋identity）以 HTTP 模擬 client 走完整流程——發起→登入核准→輪詢取得 token 對→以 access token 執行既有查詢與命令路由→refresh 換發→舊值重用觸發 family 撤銷→帳號頁撤銷另一 family。驗收：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KXFHM1AJXKMENDZMCJTFCTYW -->
- [x] 5.2 執行 npm run test:all 確認全 workspace 回歸：parity 31 項、color 16 項、twin 8 情境凍結零 diff；CLI client 零變更。驗收：全數通過。 <!-- speclink-task:tsk_01KXFHM1AJGJKR0Q210V0JHJCT -->

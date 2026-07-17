## 1. device flow typed client（TDD）

- [x] 1.1 紅（design「決策 1：device flow client 落 speclink-remote」）：新增 crates/speclink-remote/tests/device_flow.rs——以 in-process speclink-server（memory identity、tempdir store）覆蓋：initiate 回 user_code/verification_uri/interval、poll 於未核准時回 pending、測試直接對 identity store 核准後 poll 回 granted 且含 access/refresh token、瀏覽器端拒絕回 denied、逾時回 expired、refresh rotation 換新後舊 refresh credential 重用被拒（family revocation）。cargo test -p speclink-remote 確認全紅。 <!-- speclink-task:tsk_01KXQ92J9QCVP842KX1PJW3G8F -->
- [x] 1.2 綠：新增 crates/speclink-remote/src/device.rs——initiate／poll（尊重 server interval、typed 狀態）／refresh／revoke 四函式走 protocol 的 device DTOs；crates/speclink-remote/src/lib.rs 匯出。1.1 全綠。 <!-- speclink-task:tsk_01KXQ92J9QXG0FVWX3D69D5EBK -->

## 2. Keychain 與 registry（TDD）

- [x] 2.1 紅（design「決策 2：credential 唯一落點 OS Keychain，Rust 側進出」「決策 4：connection registry 檔案形狀與位置」；規格「connection registry 不含 secret 且跨重啟保留」）：src-tauri 測試覆蓋——CredentialStore trait 的 in-memory 實作 get/set/delete 逐 origin＋種類（refresh/pat）語意；registry 序列化往返斷言欄位全集（id、origin、name、lastActorDisplay）且無任何 token 欄位；同 origin 重複新增即更新顯示名；壞 JSON 歸零清單。確認全紅。 <!-- speclink-task:tsk_01KXQ92J9QQ55G1FP90TNF40W0 -->
- [x] 2.2 綠：新增 apps/desktop/src-tauri/src/credentials.rs（trait＋keyring 生產實作，service 名 speclink-desktop）與 apps/desktop/src-tauri/src/connections.rs（appConfigDir 下 connections.json 的 registry 讀寫與 connection_add/list/remove 命令雛形）；apps/desktop/src-tauri/Cargo.toml 加 keyring 與 speclink-remote/speclink-protocol 依賴。2.1 全綠。 <!-- speclink-task:tsk_01KXQ92J9QZ74VRC9YRPSC2FZX -->

## 3. root 層身分查詢 /auth/whoami（TDD）

- [x] 3.1 紅（design「決策 8：root 層身分查詢 GET /auth/whoami（server 端唯一新增）」；規格 server-device-auth「root 層 bearer 身分查詢」）：新增 crates/speclink-server/tests/auth_whoami.rs——in-process server（memory identity）覆蓋：device flow 核准取得的 access token → 200 回核准者顯示名與識別；有效 PAT → 200 且該 PAT last-used 前進；缺席、格式錯誤、已撤銷 bearer → 同一 401 permission_denied；並於 crates/speclink-remote/tests/device_flow.rs 補 whoami 案例（核准後以 access token 查得核准者顯示名、無效 bearer 得 permission_denied）。cargo test -p speclink-server -p speclink-remote 確認新案例全紅。 <!-- speclink-task:tsk_01KXQDPV22VVSHBP22FEPFDH3J -->
- [x] 3.2 綠：speclink-protocol 新增 AuthWhoamiResponse DTO（camelCase、JSON Schema）；speclink-server 新增 auth_whoami handler 與 app.rs 的 GET /auth/whoami 路由（bearer 解析與 Binding 第一步一致：spk_at_ 走 access token、其餘走 PAT、失敗同一 401、PAT touch last-used）；speclink-remote device 模組新增 whoami(base_url, bearer) 函式。3.1 全綠。 <!-- speclink-task:tsk_01KXQE63D4JPY4YM08P69Z3AFB -->

## 4. 登入／登出編排（TDD）

- [x] 4.1 紅（design「決策 5：device login 編排與瀏覽器開啟」「決策 3：登入前探測＝直接 POST /auth/device，不靠 binding capability」；規格「device login 預設與 PAT fallback」）：src-tauri 編排測試（假瀏覽器開啟器＋in-process server＋in-memory CredentialStore）——device_login 走 initiate→開啟器收到 verification URL→核准後 granted→refresh credential 入 store→/auth/whoami 身分寫回 registry；404 探測回報不支援（觸發 PAT fallback 訊號）；5xx 回報連線錯誤而非 fallback；pat_login 以 /auth/whoami 驗證後才入 store、無效 PAT 拒絕；rotation 後新 refresh credential 覆寫。確認全紅。 <!-- speclink-task:tsk_01KXQ92J9QE834AHYJ2YTAXN33 -->
- [x] 4.2 綠：實作 device_login／pat_login／logout 命令與 access token 記憶體持有；logout 落實規格「登出撤銷與移除連帶清理」，依 design「決策 6：登出與移除語意」——refresh 走 /auth/revoke 盡力撤銷、PAT 僅刪 entry 並回報提示、server 不可達不阻擋本機刪除、移除連線先登出再刪條目。4.1 全綠。 <!-- speclink-task:tsk_01KXQ92J9QJT4G2F9817E1PTDR -->

## 5. TS 接線與伺服器頁籤

- [x] 5.1 新增 apps/desktop/src/adapter/connections.ts（連線清單/新增/登入/登出/移除的 invoke 包裝，型別無任何 secret 欄位；PAT 僅作 pat_login 參數單次過境）與 apps/desktop/src/store.ts 的 connections 分片（清單、逐連線登入狀態與身分、進行中/錯誤狀態）。驗證：npm test -w apps/desktop 既有套件不受影響。 <!-- speclink-task:tsk_01KXQ92J9Q72DXQDD4WX0ANC7K -->
- [x] 5.2（規格「伺服器管理最小面」；design 之 決策 7：UI 最小面——設定頁「伺服器」頁籤）：新增 apps/desktop/src/components/ServersPanel.tsx 並掛入 apps/desktop/src/views/SettingsView.tsx 新頁籤——清單（顯示名、origin、狀態、身分）、新增表單（URL＋顯示名，用自建 Input）、登入（device 預設；收到不支援訊號就地現 PAT 輸入）、登出、移除；繁中文案。新增 apps/desktop/src/__tests__/serversPanel.test.tsx 以假 adapter 覆蓋：新增→清單即現、探測不支援→PAT 輸入現身、登入成功→顯示身分、登出→回未登入、device 拒絕→可讀狀態。全綠。 <!-- speclink-task:tsk_01KXQ92J9QKXNH4BSXSPSNCPHF -->

## 6. 驗收與衛生

- [x] 6.1 secret 衛生（規格「credential 唯一落點為 OS Keychain」）：registry 序列化測試無 token 欄位（2.1 已涵蓋則指認之）；檢視新增碼確認無 credential 落 log；TS adapter 型別檢查無 secret 欄位。驗證方式記入 PR 描述或 commit 訊息。 <!-- speclink-task:tsk_01KXQ92J9QZNJ1D9T5S9VM13X2 -->
- [x] 6.2 GUI 鐵律手動全鏈（design Implementation Contract；操作前確認使用者未在使用螢幕）：npm run dev 起本地 server → 設定頁新增 http://localhost:8080 → 真實 device login（瀏覽器 /activate 核准）→ 顯示身分；macOS 以 security find-generic-password 確認 Keychain entry；登出後 entry 消失且 server /account 的 device 清單該家族已撤；PAT fallback 實走一次（對關掉 device 端點不可行則以無效→有效 PAT 路徑驗證輸入面）；重啟 app 連線清單保留且 rotation 換發後無需重登入。 <!-- speclink-task:tsk_01KXQ92J9QW5Q7VGSYHPCY8M8H -->
- [x] 6.3 回歸：cargo test -p speclink-remote -p speclink-server、npm test -w apps/desktop、npm test -w packages/ui、cargo build --release -p speclink-desktop 全綠（重建前先關閉執行中的 exe）。 <!-- speclink-task:tsk_01KXQ92J9QT2M6JBP3E40QMTQY -->

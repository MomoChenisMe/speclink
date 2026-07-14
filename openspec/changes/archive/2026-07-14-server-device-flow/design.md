## Context

server-identity-pat 刀交付了 identity 儲存（users/memberships/invitations/PATs/sessions，SQLite schema version 守門）、session 登入（HttpOnly/Secure/SameSite=Strict cookie、同源驗證）、帳號頁與 PAT bearer 前置。藍圖 §13.3 的 Desktop 首選登入序列圖要求：Desktop 發起 device 授權、系統瀏覽器開核准頁、使用者登入核准、Desktop 輪詢取得 short-lived access token 與 rotating refresh credential、登出時撤銷 server session。wire error reason registry 是八值封閉集合（client-protocol 規格），device flow 的中間狀態（pending、slow_down）不是錯誤，不能塞進 registry。

## Goals / Non-Goals

**Goals:**

- device flow 端點在 server 側完整可用且以 typed DTO 定義——Phase 3 Desktop 只需按 DTO 對接，不回頭改 server。
- access token 短效與 refresh rotation 落地，重用即撤銷 family；全部憑證沿用 identity 儲存的 hash 落庫與即時失效語意。
- 核准動作永遠出於已登入使用者的明確確認，核准頁沿用既有 session 與同源防護。

**Non-Goals:**

- 不改 CLI client 與 speclink-remote（CLI 的 device login 屬 Phase 3 刀；本刀 e2e 以 HTTP 模擬 client）。
- 不做 Desktop/Keychain 整合（Phase 3）、不做 OIDC/SSO（後續 Phase）。
- 不做 rate limiting 基礎設施：發起端點的濫用防護只做 user code 高熵與到期，全域限流屬部署層。
- 不動 PAT 生命週期與既有帳號頁行為（只在 sessions 清單納入 device 憑證）。
- 不宣告 push transport；binding capabilities 不變。

## Decisions

### 決策 1：flow 狀態走 DTO 欄位，不擴 error registry

輪詢端點的中間與終局狀態（pending、slow_down、approved、expired、denied）以 DTO 的 status 欄位表達，HTTP 200 回應；wire error 三元組只留給真正的協定錯誤（格式不合 invalid_argument、未知 device code 回 not_found）。這保住 client-protocol 規格的八值封閉 registry——狀態機不是失敗分類。DTO（發起請求/回應、輪詢請求/回應、refresh 請求/回應）落 speclink-protocol 的 device 模組，camelCase、進既有 JSON Schema 匯出與序列化往返測試。

### 決策 2：device code 與 user code 分權責

發起回應含兩個識別：device code（高熵、只給發起的 client、用於輪詢與換 token）與 user code（短、可人工輸入、只用於核准頁比對）。核准頁憑 user code 找到待核准請求並顯示確認；核准者是當下登入的 user，核准記錄綁其身分。兩碼皆 hash 落庫、皆有到期（預設 15 分鐘）；user code 以避開易混淆字元的字母表產生。

### 決策 3：access token 短效、refresh rotation、family 撤銷

核准後輪詢換得：access token（spk_at_ prefix、預設 1 小時效期、hash 落庫、綁 user）與 refresh credential（spk_rt_ prefix、一次性）。refresh 端點以有效 refresh credential 換發新 access token 與新 refresh credential，舊 refresh 立即失效；同一 credential family 記 family id——已失效的 refresh 被再次使用即視為外洩訊號，撤銷整個 family（含現行 access token 與 refresh）。此即藍圖「short-lived access + rotating refresh credential」的具體化。

### 決策 4：access token 併入既有 bearer 前置，語意與 PAT 一致

auth.rs 的 bearer 查驗按 prefix 分流：spk_pat_ 走 PAT 表、spk_at_ 走 access token 表，其後檢查一致——hash 命中、未撤銷、未過期、所屬 user 為 active、具該 project membership，逐請求查驗無快取。停權 user 使其全部 device 憑證與 PAT 同步即時失效。錯誤分類沿用：無效類 401、非成員 403，不區分原因。

### 決策 5：device 憑證是帳號頁 sessions 的一級公民

帳號頁 sessions 清單納入 device credential families（顯示建立時間、最近 refresh、核准來源），可逐一撤銷——撤銷 family 使其 access token 與 refresh credential 即時失效。這對應藍圖「/account 管理自己的 sessions」與登出撤銷語意：Desktop 登出呼叫 refresh credential 撤銷端點，或使用者事後在帳號頁補撤銷。

### 決策 6：identity schema version 遞增並提供 migrate

device 憑證表（device 授權請求、access tokens、refresh credentials 與 family）落 identity 資料庫，schema version 由 1 遞增為 2；沿用既有守門——version 1 的資料庫由 migrate 升級，較新版本拒開。migrate 只加表不動既有資料，server-identity-pat 建立的 users/PATs/sessions 完整保留。

## Implementation Contract

- Behavior：client 發起 device 授權取得兩碼與核准 URI；使用者在瀏覽器登入後於核准頁輸入 user code 核准；client 輪詢由 pending 轉為取得 access token 與 refresh credential；以 access token 呼叫全部既有 API 路由與 PAT 等效；到期前以 refresh credential 換新；重用舊 refresh 使整個 family 失效；使用者可在帳號頁撤銷該 device session。
- Interface / data shape：POST /auth/device（發起，無需認證）→ deviceCode、userCode、verificationUri、expiresIn、interval；POST /auth/device/token（輪詢）→ status 欄位（pending、slow_down、approved、expired、denied）與核准時的 accessToken、refreshToken、expiresIn；POST /auth/refresh → 新 token 對；POST /auth/revoke（以 refresh credential 撤銷 family）；GET/POST /activate（session 保護核准頁）。DTO 全數在 speclink-protocol 的 device 模組。
- Failure modes：未知或格式不合的 device code → wire error 三元組（not_found、invalid_argument）；輪詢間隔低於宣告 interval → status 為 slow_down；授權請求逾期 → status 為 expired；使用者拒絕 → status 為 denied；已失效 refresh 重用 → family 全撤銷且該次請求回 401 permission_denied；未登入開核准頁 → 導向 /login。
- Acceptance criteria：cargo test -p speclink-server 全綠（狀態機、rotation、family 撤銷、bearer 併入、核准頁）；cargo test -p speclink-protocol 全綠（DTO schema 匯出與往返）；npm run test:all 全綠且既有凍結零 diff。

## Risks / Trade-offs

- 憑證種類增加（PAT、access、refresh）→ 以 prefix 分流與同一張檢查清單收斂複雜度；驗證邏輯單點共用。
- 無全域 rate limiting 下發起端點可被灌注記錄 → 兩碼高熵＋短到期＋未核准記錄可被排程清理；正式限流屬部署層（reverse proxy），文件註記。
- refresh family 撤銷是強制全下線 → 對外洩訊號寧可過度反應；誤傷情境（client 重試競態）由 client 以核准流程重登恢復。
- schema version 升到 2 → migrate 路徑經測試覆蓋（version 1 資料庫升級後既有資料完整），守門語意不變。

## Migration Plan

前置依賴 server-identity-pat 已歸檔。identity 資料庫 version 1 → 2 由 migrate 自動升級（只加表）；無既有 device 憑證資料要遷移。client side 零變更；Phase 3 Desktop 刀按 protocol DTO 對接。回退即回捨本 change——version 2 資料庫對舊 binary 會被守門拒開，回退需同時還原資料庫檔（尚無正式部署，可接受）。

## Open Questions

（無）

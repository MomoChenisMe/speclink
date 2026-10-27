## Context

speclink-server 目前的認證在 auth.rs：Authorization bearer 對組態檔 tokens 段的明文靜態表逐一比對，命中即得 actor；組態載入在 config.rs（fail closed），binding 前置已重用 host 的裁決。全部路由是 JSON API，無任何 HTML 服務、無 session、無密碼學依賴（workspace 現有 sha2）。平台藍圖 §13.3 定義目標模型：invitation → 本機密碼 → /account 自助 PAT（明文一次、只存 prefix+hash、到期與撤銷、last-used）、停權/降權即時生效；§9.2 約束瀏覽器認證用 same-origin HttpOnly cookie、PAT 不進 URL、localStorage 或 log。

## Goals / Non-Goals

**Goals:**

- 帳號、邀請、session、PAT 的完整生命週期落在 server 自有的 identity 儲存，bootstrap token 組態退場。
- 一般使用者不碰主機也不找 Admin 換發：瀏覽器完成受邀、登入、PAT 自助建立與撤銷。
- 憑證安全底線：密碼 argon2、PAT/邀請/session 憑證只存 hash、PAT 明文一次、cookie 全套安全屬性、失效即時。

**Non-Goals:**

- 不做 device authorization flow、access/refresh token（server-device-flow 刀）。
- 不做 /setup 首次啟動流程、/admin Web UI、audit log、backup（後續子刀）；admin 旗標欄位先落 schema，本刀唯一授予途徑是 invite 子命令。
- 不做 OIDC/SSO（藍圖明示後續 Phase）、不做 email 寄送（invite URL 由建立者自行遞送）、不做密碼重設流程（本刀以重新邀請覆蓋）。
- 不做 project 內細粒度 role（reader/writer 分級屬 admin 子刀）；本刀的授權判準是 membership 有無。
- 不動 CLI client、speclink-remote、speclink-protocol：貼 PAT 的 remote auth 流程對 client 完全透明。

## Decisions

### 決策 1：identity 儲存獨立資料庫檔，不混 TeamStore

identity 資料落 server 自有的 SQLite 檔（組態 identity 段宣告路徑），與 TeamStore 的資料庫分檔：TeamStore driver 的 schema 版本守門拒絕外來表，混檔會迫使兩個不相關的 schema 綁死同一版本序。identity 儲存以 trait 抽象（users、memberships、invitations、pats、sessions 的讀寫），SQLite 為正式實作、in-memory 變體僅供路由測試；沿用 sqlite-team-store 的守門原則——meta 表記 schema version，未知較新版本或非 speclink identity schema 拒開回明確錯誤，不靜默初始化非空的陌生資料庫。

### 決策 2：憑證儲存一律 hash，PAT 帶可辨識 prefix

密碼以 argon2id 儲存（argon2 依賴只進 speclink-server）。PAT 形如 spk_pat_ 接高熵隨機段：資料庫只存 token id、顯示用 prefix（前 12 字）、SHA-256 hash、名稱、到期、撤銷時戳與 last-used——PAT 本身高熵，不需慢速 hash，SHA-256 查表即可。邀請 token 與 session id 同樣高熵隨機、只存 hash。任何 API 或頁面都不能讀回明文；PAT 明文只出現在建立回應一次。

### 決策 3：invite 子命令是本刀唯一的管理入口

speclink-server binary 增 invite 子命令：以 --config 找到 identity 資料庫，直接建立邀請（--email、--display、--project 可重複、--admin、--expires-in-days 預設 7）並在 stdout 輸出一次性 invite URL。主機檔案系統存取即是管理權——這是 §13.2 的 headless server CLI 路徑，第一位使用者（含 admin）由運維者以此建立，不需要先有 Web UI。重複 email（已有 active user 或未過期邀請）拒絕，避免影子帳號。

### 決策 4：Web 入口是 server-rendered 表單，session cookie 全套安全屬性

invite 接受頁、/login、/logout、/account 與 PAT 建立/撤銷全部是嵌入 binary 的 server-rendered HTML 表單（無 JS 框架、無外部資源）。session cookie：HttpOnly、Secure、SameSite=Strict、路徑限定；全部變更型 POST 驗證 Origin/Referer 與 server 設定的對外 URL 同源，不符回 403。登入失敗訊息統一「帳號或密碼不正確」，不洩漏 email 是否存在；邀請已用或過期回同一「邀請無效」頁，不區分原因。/logout 撤銷 server 端 session 記錄——cookie 清除只是輔助，撤銷以資料庫為準。

### 決策 5：API bearer 驗證逐請求查 identity 儲存，即時生效

auth.rs 的 token 查驗改為：bearer 值 SHA-256 後查 PAT 表，命中再逐項檢查——未撤銷、未過期、所屬 user 為 active、user 是 URL project 的 member——全過才得 actor，並非同步更新 last-used。無快取：停權、撤銷、降權（移除 membership）在下一個請求立即生效，這是 §13.3「role 被降低或 user 停權後既有 PAT 立即失效」的直接實作。錯誤分類：token 無效/過期/撤銷/停權回 401 permission_denied（不區分原因，避免探測）；token 有效但非該 project 成員回 403 permission_denied。

### 決策 6：組態 tokens 段退場，測試播種走 identity 儲存

config.rs 移除 tokens 段、新增 identity 段（資料庫路徑；測試組態可宣告 memory）。既有整合測試與 e2e 的播種 helper 改為經 identity 儲存 trait 建 user、membership 與 PAT（測試持 AppState 直接呼叫，不繞 HTTP）。組態含未知段或 identity 段形狀不合仍是啟動 fail closed；沿用既有 ConfigError 錯誤報告格式。

## Implementation Contract

- Behavior：運維者以 invite 子命令產生 URL 交給成員；成員開啟 URL 設密碼、登入 /account、建立 PAT 並貼進 CLI 的既有 remote auth 流程，全部 remote 動詞照常運作；成員在 /account 撤銷 PAT 後，下一個帶該 PAT 的請求即 401。
- Interface / data shape：組態檔 identity 段（sqlite 路徑或 memory）；invite 子命令的參數與 stdout URL；Web 路由——GET/POST invite 接受頁、GET/POST /login、POST /logout、GET /account、POST /account/tokens（建立，回應頁顯示明文一次）、POST /account/tokens/{id}/revoke；API bearer 語意不變（Authorization: Bearer spk_pat_...）。
- Failure modes：邀請已用/過期/未知 → 同一「邀請無效」頁（HTTP 404）；登入失敗 → 統一錯誤訊息（HTTP 401 頁）；未登入訪問 /account → 導向 /login；POST Origin 不符 → 403；API 側 token 無效類 → 401 permission_denied、非成員 → 403 permission_denied；identity 資料庫不可開（版本過新、schema 陌生、路徑不可寫）→ 啟動失敗印原因。
- Acceptance criteria：cargo test -p speclink-server 全綠（identity 儲存、invite 子命令、web 入口、bearer 接線、e2e）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- 兩個 SQLite 檔（TeamStore＋identity）→ 備份子刀須涵蓋兩者；換取 schema 演進互不綁死與 driver 契約純淨。
- 逐請求查 identity 資料庫 → single-node 定位下每請求多一次本機 SQLite 讀，可接受；換取失效即時性，不建快取失效協定。
- 無密碼重設與 email 驗證 → 運維者以重新邀請處理遺失密碼；正式帳號治理屬 admin 子刀。
- server-rendered 表單無 JS → 體驗陽春但零前端依賴、CSP 直接收緊；Desktop/admin 的完整 UI 各有後續刀。

## Migration Plan

尚無正式部署，遷移範圍即 repo 內測試資產：測試組態移除 tokens 段改 identity memory 段、播種 helper 改建 user+PAT、e2e 的 token 來源改 PAT 明文。實作順序：identity 儲存 → invite 子命令 → web 入口 → bearer 接線與組態切換（此步之後舊 tokens 組態即失效，同一 change 內一次完成不留雙軌）。回退即回捨本 change；identity 資料庫檔可直接刪除重建。

## Open Questions

（無）

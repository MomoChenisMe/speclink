## MODIFIED Requirements

### Requirement: 邀請一次性且到期失效
<!-- BEFORE: 有效 invite URL 呈現 server-rendered 密碼表單，提交後建立 active user 並耗用邀請。 -->

邀請 SHALL 由 server binary 的 invite 子命令於主機上建立（email、顯示名、指派的 project memberships、可選 admin 旗標、到期時限），並輸出一次性 invite URL；對已有 active user 或未過期邀請的 email SHALL 拒絕重複建立。開啟有效 `/invite/:token` SHALL 由 browser API 回傳設定密碼所需的非祕密邀請摘要；同源提交後 SHALL 原子地建立 active user（含指派 memberships）並耗用邀請，接著建立該 user 的 Web session。成功回應 SHALL 回傳由 Server 裁決的 `destination`：admin invitation 為 `/admin`，一般 invitation 為 `/account`。已用、過期或未知的邀請 token SHALL 得到相同「邀請無效」狀態與公開訊息，SHALL NOT 區分原因，且 SHALL NOT 建立 session。

#### Scenario: 邀請走完即建立帳號並登入

- **WHEN** 以 invite 子命令對新 email 建立含一個 project membership 的邀請，開啟 URL 設定密碼並提交
- **THEN** user 建立為 active 且具該 membership，Web session 已設定並導向 `/account`；同一 URL 再開啟得到「邀請無效」

#### Scenario: Admin 邀請進入管理首頁

- **WHEN** 帶 admin 旗標的有效邀請完成密碼設定
- **THEN** user 與 Web session 建立成功，Server 回 `destination: "/admin"`，SPA 進入管理首頁

#### Scenario: 過期邀請不可用

- **WHEN** 開啟已過到期時限的邀請 URL
- **THEN** 回應與已用邀請相同的「邀請無效」狀態；不建立 user 或 session

#### Scenario: 重複 email 拒絕

- **WHEN** 對已有 active user 的 email 執行 invite 子命令
- **THEN** 子命令以非零 exit code 拒絕並說明原因；不建立邀請

#### Scenario: 建立 session 失敗不偽裝成已登入

- **WHEN** user 與邀請交易成功後 Web session 建立失敗
- **THEN** Server 回不含內部細節且指示可重試登入的 500 recovery error，SHALL NOT 回成功 destination

### Requirement: 本機密碼登入與 session 安全屬性
<!-- BEFORE: 登入取得 session 後固定使用 server-rendered 帳號頁，未登入訪問帳號頁導向登入頁。 -->

一般使用者 SHALL 能經 browser JSON API 以 email 與本機密碼登入取得 session；密碼 SHALL 以 argon2id 儲存。Session cookie SHALL 具 HttpOnly、Secure 與 SameSite=Strict 屬性；全部 browser mutation SHALL 驗證同源，不符 SHALL 回 403。登入失敗 SHALL 回相同狀態與統一錯誤訊息，SHALL NOT 洩漏 email 是否存在。登出 SHALL 撤銷 server 端 session 記錄；被撤銷或過期的 session 後續 browser API 請求 SHALL 回 401。

登入成功 destination SHALL 由 Server 依序裁決：有效 device `userCode`、安全 `returnTo`、角色 home。安全 `returnTo` SHALL 只接受以單一 `/` 開頭、無 scheme 或 authority，且首段為 `/account`、`/activate` 或 `/admin` 的路徑；一般成員的 `/admin` destination SHALL 回 403。未登入訪問受保護 SPA route SHALL 導向 `/login?returnTo=...`，且只保留通過同一白名單的站內路徑。

#### Scenario: 登入失敗訊息不洩漏帳號存在性

- **WHEN** 分別以不存在的 email 與存在但密碼錯誤的 email 提交登入
- **THEN** 兩者的回應狀態與錯誤訊息文字相同，且皆不建立 session

#### Scenario: 登出後 session 立即失效

- **WHEN** 登入後執行登出，再以同一 cookie 請求 account browser API
- **THEN** 請求回 401；server 端該 session 記錄已標記撤銷，SPA 導向登入頁

#### Scenario: 角色 home 由 Server 回傳

- **WHEN** admin 與一般成員各自在沒有 device code 與 `returnTo` 時登入
- **THEN** admin 成功回應的 destination 為 `/admin`，一般成員為 `/account`

#### Scenario: 安全 returnTo 優先於角色 home

- **WHEN** admin 以 `/account` 作為 `returnTo` 完成登入
- **THEN** Server 驗證站內路徑後回 `destination: "/account"`

#### Scenario: 外部 returnTo 被忽略

- **WHEN** 使用者以 `https://evil.example/path` 作為 `returnTo` 完成登入
- **THEN** Server 不回外部目的地，改回該使用者的角色 home

## ADDED Requirements

### Requirement: 帳號 browser API 保持憑證祕密邊界

登入使用者 SHALL 能經 `/api/speclink/v1/web/account` 讀取 user、PAT metadata、Web sessions 與 device families，並建立／撤銷自己的 PAT、登出 Web session、撤銷 device family。讀取 payload SHALL 僅含呈現與 eligibility 所需 metadata，SHALL NOT 包含 PAT hash、password hash、refresh credential 或可重播的 session secret。PAT 建立回應 SHALL 只在該次 `{data}` 內回傳 plaintext；後續讀取 SHALL 僅回 prefix、名稱、到期、撤銷時戳與 last-used。所有 mutation SHALL 驗證同源與 active session。

#### Scenario: PAT 明文只在建立回應出現

- **WHEN** 使用者經 browser API 建立 PAT，接著重新讀取 account summary
- **THEN** 建立回應包含 plaintext；summary 只含 prefix 與 metadata，沒有途徑再次取得 plaintext

#### Scenario: 撤銷 device family 即時生效

- **WHEN** 使用者從帳號頁撤銷一個仍有 active refresh credential 的 device family
- **THEN** family 內 refresh credential 立即失效，後續 account summary 顯示該 family 已撤銷且不回傳 credential

#### Scenario: Account summary 不外洩其他使用者資料

- **WHEN** 一般成員呼叫 account summary
- **THEN** 回應只含該 session user 自己的 user、PAT、Web session 與 device family metadata

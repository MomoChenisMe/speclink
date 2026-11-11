## Context

用戶端登入的現況：

- `speclink-remote` 已經是 CLI 與 Speclink Desktop 共用的地方：金鑰圈的存取（服務名稱 `speclink-desktop`，帳號是「種類:位址」，種類有 `refresh`、`pat`、`bearer`）、憑證檔、換新 token 與它的跨程序鎖、裝置登入的五個 HTTP 呼叫、一個由 CLI 使用的裝置登入迴圈（`login::device_login`，顯示與開瀏覽器經一個 trait 交給呼叫端）。
- Speclink Desktop 沒有用那個迴圈。它的 Rust 端每次只輪詢一下，計時器、間隔加倍與逾時判斷都在前端的 TypeScript。
- 兩邊不一致的地方有十處，主要的四處：存取金鑰的存放位置（CLI 是憑證檔，Desktop 是金鑰圈）；位址怎麼變成金鑰圈的鍵（CLI 直接切字串，Desktop 會轉小寫並檢查 scheme）；登出清掉什麼（Desktop 不清 access token 的快取）；驗證存取金鑰用哪個端點（CLI 用專案底下的 whoami，Desktop 用根目錄的）。
- 選擇登入方式沒有協商：先對裝置登入的端點發請求，收到 404 或 405 才改問存取金鑰。
- 整個 repo 沒有 OAuth 或 PKCE 的程式。

限制：

- 討論的裁定：speclink 不定義也不實作登入的伺服器端。用戶端只依公開標準：RFC 8252（桌面程式的授權碼流程）、RFC 7636（PKCE）、RFC 8414（登入服務的標準文件）。
- 已查證 RFC 8252 的條文：7.3 節規定授權伺服器必須允許本機回呼位址在請求時指定任意埠，回呼位址的形式是 `http://127.0.0.1:{port}/{path}`；8.3 節不建議用 `localhost`；8.1 節說明公開的桌面用戶端必須用 PKCE。已查證 RFC 8414：標準文件的位置是把 `/.well-known/oauth-authorization-server` 插在主機與路徑之間；用戶端必須確認文件裡的 `issuer` 和用來組網址的值完全相同。
- refresh token 不能落在明文檔（既有規格的要求）。
- Desktop 的規則：access token 與 refresh token 不能出現在 TypeScript 看得到的狀態。

## Goals / Non-Goals

**Goals:**

- 用戶端能對任何提供標準文件的登入服務完成瀏覽器登入，並在 token 過期時自己換新。
- 用戶端在登入前知道 Server 支援哪些方式。
- 探索、選擇、三種登入、登出只有一份實作，CLI 與 Desktop 共用。
- 連到今天的官方 server 時，行為、輸出與 exit code 不變。

**Non-Goals:**

- 任何登入的伺服器端。
- Server 回答探索與 API 根的 whoami（變更 host-kit-client-door）。
- 自訂 URL scheme 的回呼、RFC 8628 格式的裝置登入、動態用戶端註冊。
- 搬移既有憑證檔的內容。

## Decisions

### D1: 探索的線上型別與路徑

`speclink-protocol` 新增 `auth.rs`：

```rust
pub struct AuthMethodsResponse { pub methods: Vec<String>, pub browser: Option<BrowserLogin> }
pub struct BrowserLogin { pub issuer: String, pub client_id: String, pub scopes: Vec<String> }
```

JSON 是 camelCase：`{ "methods": ["browser", "device", "token"], "browser": { "issuer": "...", "clientId": "...", "scopes": ["openid"] } }`。`scopes` 可以省略，省略時是空的。`browser` 物件只在 `methods` 含 `browser` 時有意義。

路徑：`GET {origin}/api/speclink/v1/auth-methods`，不帶 token。

用戶端的讀法：

- 200 且能解析 → 用它。不認得的方式名稱略過，所以之後新增方式不會讓舊用戶端壞掉。
- 404 或 405 → 「沒有宣告」，走既有的行為（D2）。
- `methods` 含 `browser` 但沒有 `browser` 物件、或 `issuer`、`clientId` 是空的 → 把 `browser` 當成沒有宣告，其餘照用。
- 其他狀態碼、連不上 → 錯誤，不猜。

為什麼需要 `clientId`：授權碼流程的每個請求都要帶用戶端識別，而它是宿主在自己的登入服務登記的，speclink 不能寫死。

- **否決：用握手回應裡既有的 `authentication` 欄位** — 握手要先帶 token。
- **否決：把授權網址與 token 網址直接放進這個回應** — 登入服務已經有標準文件，放兩份會不同步；只給發行者網址，其餘從標準文件取得。

### D2: 選擇登入方式

共用的函式 `choose(methods, preference)`：

| 情況 | 結果 |
|---|---|
| 使用者指定了方式，Server 有宣告它（或沒有宣告任何東西） | 用指定的 |
| 使用者指定了方式，Server 有宣告但不含它 | 錯誤：這台 Server 不支援這個方式，並列出它支援的 |
| 沒有指定，Server 有宣告 | 依固定順序取第一個有宣告的：`browser`、`device`、`token` |
| 沒有指定，Server 沒有宣告（404 或 405） | 既有的行為：先試裝置登入，裝置登入的端點回 404 或 405 才改用存取金鑰 |

順序是用戶端固定的，不看回應裡的排列。

### D3: 瀏覽器登入的步驟

放在 `speclink-remote` 的 `oauth.rs`（標準文件、授權網址、換 token、換新）與 `loopback.rs`（本機的回呼監聽）。

1. **先確認金鑰圈能用。** 不能用就結束並說明，不開瀏覽器。理由和裝置登入相同：refresh token 不落明文檔。
2. **取得標準文件。** 先試 RFC 8414 的位置（`/.well-known/oauth-authorization-server` 插在主機與路徑之間）；回 404 時改試 OpenID Connect 的位置（發行者網址後面接 `/.well-known/openid-configuration`）。檢查：文件的 `issuer` 和設定的發行者網址完全相同；有 `authorization_endpoint` 與 `token_endpoint`；`code_challenge_methods_supported` 有列出時必須含 `S256`（沒有這個欄位時照樣送 S256，因為有些常見的登入服務不列它但支援）。任一項不符就結束並說明是哪一項。
3. **網址的 scheme。** 發行者、授權網址、token 網址必須是 https；主機是 `127.0.0.1`、`::1` 或 `localhost` 時允許 http（開發用）。
4. **開監聽。** 綁 `127.0.0.1` 的 0 號埠，由作業系統給一個空的埠。回呼位址是 `http://127.0.0.1:<埠>/callback`。
5. **產生 PKCE 與 state。** verifier 是 32 個隨機位元組的 base64url（43 個字元）；challenge 是它的 SHA-256 的 base64url；state 是另外 32 個隨機位元組。隨機來源是作業系統。
6. **組授權網址**：`response_type=code`、`client_id`、`redirect_uri`、`code_challenge`、`code_challenge_method=S256`、`state`，有 scope 時以空白相連。
7. **顯示並開瀏覽器。** 經呼叫端提供的介面顯示網址並嘗試開啟系統瀏覽器；開不了也繼續等，使用者可以自己貼上。
8. **等回呼。** 只接受對 `/callback` 的 GET；其他路徑回 404 並繼續等。`state` 不符的請求回 400 並繼續等（不結束，避免別的程式用一個假請求就中止登入）。回呼帶 `error` 時結束，分成「使用者拒絕」（`access_denied`）與其他錯誤。逾時是 300 秒。呼叫端回報取消時結束。
9. **回應瀏覽器。** 成功時回一頁純文字的 HTML：「登入完成，可以關閉這個視窗」（中英各一句）；失敗時回一頁說明失敗。頁面不含任何 token 或授權碼。
10. **換 token。** 對 token 網址送表單：`grant_type=authorization_code`、`code`、`redirect_uri`、`client_id`、`code_verifier`。不帶用戶端密鑰。
11. **存檔。** 在既有的換新鎖裡寫入（D4）。
12. **查身分。** 用拿到的 access token 問 Server（D6），回傳顯示名稱。

失敗後的狀態：第 11 步之前的任何失敗，本機沒有新增或改變任何憑證。第 11 步是鎖裡的兩筆寫入（refresh 條目、access token 快取），先寫 refresh 條目；第二筆失敗時留下的是「有 refresh、沒有快取」，下一次使用時會換新一次，這是可以接受的半套。第 12 步失敗時憑證已經存好，回報「已登入但無法取得身分」，不清掉憑證。

- **否決：用 `localhost` 當回呼主機** — RFC 8252 的 8.3 節不建議：它可能被解析到別的位址。
- **否決：固定的埠** — 會和別的程式衝突；標準要求登入服務接受任意埠。
- **否決：加入 OAuth 的函式庫** — 用到的只有兩個表單請求與一個 SHA-256；`speclink-remote` 已經有 HTTP 用戶端。新增的直接相依是 sha2、base64、getrandom，三個都已經在 lock 檔裡。
- **否決：標準文件沒列 `S256` 時拒絕** — 會擋掉不列這個欄位的常見登入服務；PKCE 仍然照送，不支援的服務會在換 token 時自己拒絕或忽略。

### D4: 瀏覽器登入的憑證存在既有的 refresh 條目

金鑰圈的 `refresh:<位址>` 條目今天的值是一個字串：裝置登入的 refresh token。瀏覽器登入把同一個條目的值寫成 JSON：

```json
{ "kind": "oauth", "refreshToken": "...", "tokenEndpoint": "...", "clientId": "...", "revocationEndpoint": "..." }
```

- `refreshToken` 可以是 null（登入服務沒有發 refresh token）。
- `revocationEndpoint` 可以省略。
- 讀取時：值能解析成 `kind` 為 `oauth` 的 JSON 物件就是瀏覽器登入的憑證；否則照舊當成裝置登入的 refresh token。所以既有的條目不用搬移。
- access token 連同到期時刻寫進既有的 `bearer:<位址>` 快取，格式不變。回應沒有 `expires_in` 時當成 300 秒。

這樣憑證解析的階梯仍是四層，`credentialSource` 的值域不變，「共用一個條目、換新要排隊」的既有保證直接涵蓋瀏覽器登入。

一個位址同時只有一種登入：任何一種登入成功時，在同一把鎖裡清掉這個位址其他種類的條目（瀏覽器或裝置登入成功時清掉存取金鑰的條目；存取金鑰登入成功時清掉 refresh 與快取）。

- **否決：新增一種金鑰圈條目與第五層** — 階梯、`credentialSource`、登出、換新的鎖都要各加一個分支，而兩種 refresh 的生命週期其實一樣。

### D5: 換新

`refresh.rs` 的換新依條目的種類分流，鎖、快取、「拿到鎖後重讀快取」的邏輯不變：

- 裝置登入的條目：照舊呼叫 Server 的換新端點。
- 瀏覽器登入的條目：對 `tokenEndpoint` 送表單 `grant_type=refresh_token`、`refresh_token`、`client_id`。回應有新的 refresh token 時覆寫條目，沒有時沿用舊的。
- 被拒絕（HTTP 400 且 `error` 是 `invalid_grant`，或 401）→ 和今天「整族被撤銷」相同：清掉 refresh 條目與快取，提示重新登入。
- 連不上或 5xx → 「暫時無法換新」，不清憑證。
- 條目的 `refreshToken` 是 null → 快取過期後直接當成被拒絕。

動詞收到 401 時的重試規則不變（來源是金鑰圈的 refresh 條目才換新並重試一次）。

### D6: 登入後查身分

共用的函式先問 `GET {origin}/api/speclink/v1/whoami`（帶 token）。回 404 或 405 時改問既有的 `GET {origin}/auth/whoami`。兩者的回應形狀相同（既有的 `AuthWhoamiResponse`）。

存取金鑰的驗證也改用這個函式，CLI 與 Desktop 一致。今天 CLI 用的是專案底下的 whoami，它要求使用者是該專案的成員；改用後，金鑰有效但不是成員的人能登入成功，之後執行動詞時才收到權限錯誤。`speclink auth status` 仍然用專案底下的 whoami，所以它顯示的 repo 驗證結果不變。

API 根的 `whoami` 由變更 host-kit-client-door 的入口提供。在那之前官方 server 只有後者，用戶端一律走到退路，行為和今天 Desktop 相同。

### D7: 一份登入程式

`speclink-remote::login` 成為唯一的實作，對外：

- `discover(origin) -> Result<Discovery, RemoteError>`（D1）
- `choose(&Discovery, Option<Method>) -> Result<Plan, ChooseError>`（D2）
- `login(origin, plan, &dyn CredentialStore, config_dir, &dyn LoginIo) -> LoginOutcome`：依計畫跑瀏覽器登入、裝置登入，或回報「需要存取金鑰」。
- `pat_login(origin, token, &dyn CredentialStore, lock_dir, fallback_dir: Option<&Path>) -> LoginOutcome`：驗證後存入金鑰圈；金鑰圈不能用時存入 `fallback_dir` 的憑證檔。`fallback_dir` 是 `None` 時（Desktop 的憑證只在金鑰圈）回報「金鑰圈不能用」。
- `logout(origin, &dyn CredentialStore, credentials_file_dir: Option<&Path>) -> Result<LogoutOutcome, String>`：有裝置登入的 refresh 時呼叫 Server 的撤銷端點；有瀏覽器登入的 refresh 且條目帶撤銷網址時對它送 RFC 7009 的撤銷請求；兩者失敗都只回報警告。之後清掉 refresh、快取、存取金鑰三個條目，有 `credentials_file_dir` 時（CLI）也清掉憑證檔的條目。這個位址沒有任何本機憑證時什麼都不動，回報「沒有登入」。
- `normalize_origin(input) -> Result<String, String>`：從 Desktop 搬來的規則（scheme 與主機轉小寫、只接受 http 與 https、去掉路徑）。金鑰圈的鍵與憑證檔的鍵都用它的結果。

`LoginIo` 取代既有的 `DeviceLoginIo`：

```rust
pub trait LoginIo {
    fn phase(&self, phase: LoginPhase);          // 顯示用：等待瀏覽器、等待裝置核准（含碼、網址、期限）
    fn open_browser(&self, url: &str) -> bool;
    fn sleep_secs(&self, secs: u64);
    fn cancelled(&self) -> bool;
}
```

`LoginOutcome` 的種類：成功（顯示名稱與 handle；CLI 用存取金鑰登入時印的是「名稱 (@handle)」）、成功但查不到身分、被拒絕、逾時、取消、這台 Server 不支援、需要存取金鑰、金鑰圈不能用、錯誤（訊息）。

各端怎麼用：

- **CLI**：`cmd_auth` 依旗標組出偏好，呼叫 `discover`、`choose`、`login`。`LoginIo` 的實作把階段印到 stdout、用既有的三個平台指令開瀏覽器、`cancelled` 一律回假（Ctrl-C 直接結束程序）。「需要存取金鑰」時印出既有的指引並以非 0 結束。
- **Desktop**：Tauri 的 `connection_login_start` 在背景執行緒跑同一組函式；`LoginIo` 的實作把階段經 Tauri 事件送到前端、用既有的 opener 外掛開瀏覽器、`cancelled` 讀一個由 `connection_login_cancel` 設定的旗標。既有「先試一次安靜換新」的捷徑保留，放在 `connection_login_start` 呼叫 `login` 之前。前端的 store 只做一件事：收到階段就更新 `ConnectionPhase`；計時器、間隔加倍、逾時判斷都刪掉。倒數仍在前端依階段帶來的期限顯示。
- 每個 Tauri command 只做一行委派；背景執行緒與旗標的管理放在 `connections.rs` 既有的連線狀態旁。

裝置登入迴圈的規則沿用 CLI 那一份（收到 slow_down 時間隔加 5 秒、累計等待超過期限就逾時），Desktop 改用後不再是「間隔加倍」。

- **否決：保留 Desktop 前端的輪詢，只共用 HTTP 呼叫** — 瀏覽器登入的等待是一個阻塞的監聽，沒辦法切成「每次問一下」；兩種登入若一個在 Rust 一個在前端，取消與逾時就有兩套。

### D8: CLI 的旗標

`auth login` 的四個旗標 `--browser`、`--device`、`--pat`、`--token-stdin` 互斥（clap 的衝突設定；錯誤訊息與 exit code 是 clap 既有的用法錯誤）。對照：

| 輸入 | 行為 |
|---|---|
| 沒有旗標、互動的終端機 | `discover` → `choose(None)` → `login` |
| 沒有旗標、不是互動的終端機 | 既有行為：非 0，指引 `--token-stdin`，不發任何請求 |
| `--browser` | `discover` → `choose(browser)`；Server 沒有 `auth-methods` 時錯誤：這台 Server 沒有宣告瀏覽器登入 |
| `--device` | `discover` → `choose(device)` |
| `--pat`、`--token-stdin` | 不探索；既有的提示與輸出不變；存放位置依 D7 |

`--browser` 是唯一「Server 沒有宣告就不能用」的方式，因為沒有宣告就沒有發行者網址。

stdout 的內容：瀏覽器登入印「Open this URL to sign in:」加授權網址，成功後印既有的登入成功訊息與身分。裝置登入的輸出不變。

### D9: Desktop 的畫面

- `ConnectionPhase` 新增 `awaitingBrowser { url, expiresAt }`。
- `connectionLogin.tsx` 新增 `AwaitingBrowser`：一句說明、可複製的網址、倒數、取消。沿用既有的 `AwaitingApproval` 的版面與 `@speclink/ui` 的 `Button`、`CopyButton`；不新增共用元件。
- 伺服器頁的連線列與工作區選擇器的 Server 步驟，在這個階段就地顯示它，位置和等待裝置核准相同。
- 伺服器頁的「登入方式」卡改成三列：瀏覽器登入（伺服器支援時優先）、裝置登入、存取金鑰。仍然只是說明，沒有動作。
- 文案加在既有的訊息檔，中英各一份。

授權網址會經事件送到前端顯示。它含 state 與 challenge，不含 verifier、授權碼或任何 token，所以不違反「TS 看不到憑證」的規則。

## Implementation Contract

**行為**

- 對回 404 或 405 於 `auth-methods` 的 Server：CLI 與 Desktop 的登入流程、CLI 的 stdout、stderr 與 exit code 和這個變更之前相同。
- 對宣告 `["browser", "device"]` 的 Server：不指定時走瀏覽器登入；`--device` 走裝置登入；`--pat` 不探索，照常進行。
- 瀏覽器登入成功後：金鑰圈的 refresh 條目是 D4 的 JSON；快取條目有 access token 與到期時刻；憑證檔沒有變動；`auth status --json` 的 `credentialSource` 是 `keychain_refresh`。
- access token 過期後的第一個動詞：對 token 網址換新一次並成功，使用者看不到提示；兩個程序同時遇到時只換新一次。
- 存取金鑰登入：有金鑰圈時寫 `pat` 條目、憑證檔不變；沒有金鑰圈時寫憑證檔。
- 登出後：這個位址的 refresh、快取、存取金鑰三個條目與憑證檔的條目都不存在。
- CLI 用存取金鑰登入後，Desktop 對同一個位址顯示已登入；反過來也成立。

**介面與資料形狀**

- 線上：`GET /api/speclink/v1/auth-methods` 的回應（D1）。
- Rust：`speclink_protocol::auth::{AuthMethodsResponse, BrowserLogin}`；`speclink_remote::login::{discover, choose, login, pat_login, logout, normalize_origin, LoginIo, LoginPhase, LoginOutcome, Method}`。`DeviceLoginIo` 與 `DeviceLoginOutcome` 移除，由上面的取代。
- 金鑰圈：`refresh:<位址>` 的值多一種形狀（D4）。
- Tauri：`connection_login_start { origin, method? }`、`connection_login_cancel { origin }`、事件 `connection-login-phase`。既有的 `device_login_start`、`device_login_observe` 移除；`pat_login`、`logout` 保留名稱，內部改成委派。
- CLI：`auth login` 的 `--browser`、`--device`。

**失敗的樣子**

| 情況 | CLI | Desktop |
|---|---|---|
| 使用者在登入服務拒絕 | 非 0，「被拒絕」 | 連線列顯示已拒絕 |
| 300 秒內沒有回呼 | 非 0，「逾時」 | 顯示逾時 |
| 取消 | 不適用 | 回到未登入，沒有新增憑證 |
| 標準文件的 `issuer` 不符、缺端點、不含 S256 | 非 0，指出哪一項 | 顯示錯誤與同一句說明 |
| 發行者不是 https（也不是本機） | 非 0 | 顯示錯誤 |
| 金鑰圈不能用 | 非 0，指引 `--pat` 或 `SPECLINK_TOKEN` | 顯示錯誤 |
| Server 有宣告但不含指定的方式 | 非 0，列出支援的方式 | 不適用（Desktop 不指定） |
| `auth-methods` 回 5xx 或連不上 | 非 0，連線錯誤 | 顯示連線錯誤，不進入存取金鑰的輸入 |

**驗收**

- `cargo test -p speclink-protocol`：新型別的 JSON 形狀（camelCase、`scopes` 省略時是空的）。
- `cargo test -p speclink-remote --test it`：用假的 Server 與假的登入服務涵蓋 D1 到 D7；既有的 `device_login_flow`、`credential_ladder`、`refresh_lock`、`reauth_retry` 不改斷言就通過（只改呼叫的函式名稱）。
- `cargo test -p speclink-cli --test it auth_login_methods::`：旗標的互斥，以及不會存憑證的各旗標錯誤路徑（binary 會碰到真的 OS 金鑰圈與瀏覽器，存憑證的情境不在這一層）。
- `cargo test -p speclink-cli --bin speclink verbs::connection::`：用記憶體金鑰圈測各旗標的登入路徑、存取金鑰的存放位置、登出、對沒有 `auth-methods` 的 Server 輸出不變。
- `cargo test -p speclink-desktop`：Tauri 端的委派與取消。
- `npm test -w apps/desktop`：三種階段的畫面、取消、登入方式卡三列。

**範圍**

- 在範圍內：上面列的介面；CLI 與 Desktop 改用共用的登入。
- 不在範圍內：`speclink-server`、`speclink-host`、任何 Server 端的路徑；使用者文件。

## Risks / Trade-offs

- [回歸對照] 對官方 server 的登入不能變。→ 既有的 CLI 與 remote 整合測試不改斷言；新增一個測試明確斷言「`auth-methods` 回 404 時，裝置登入的 stdout 與今天逐位元相同」。
- [跨平台] 本機監聽在 Windows 第一次綁定時防火牆不會跳出詢問，因為只綁 `127.0.0.1`；三個平台都用 0 號埠。開瀏覽器沿用既有的三套指令。→ 回呼監聽的測試在三個平台的 CI 都跑；逾時用可注入的時鐘，不讓測試真的等 300 秒。
- [無桌面環境的 Linux] 沒有金鑰圈時瀏覽器登入與裝置登入都不能用，只有存取金鑰（存憑證檔）與 `SPECLINK_TOKEN`。→ 和今天相同，訊息沿用。這個建置只在 macOS 與 Windows 編進原生金鑰圈；其他平台上 keyring crate 用的是不保存資料的 mock（寫入成功但資料隨即消失），`KeyringCredentialStore` 把它當成「金鑰圈不能用」，存取金鑰才會落到憑證檔。
- [回呼被同機的其他程式搶先] 攻擊者要先知道隨機的 state 才能讓回呼被接受，而且拿不到 verifier。→ state 與 PKCE 都是必要的，不提供關閉的選項。
- [新舊版本混用] 舊版把 JSON 當成 refresh token 去換新，被拒絕後清掉條目。→ 只在宿主啟用瀏覽器登入時發生；寫進相容性影響。
- [位址正規化改變金鑰圈的鍵] 含大寫主機名稱的既有登入要重登一次。→ 寫進相容性影響；不做搬移。
- [Desktop 的輪詢規則改變] 裝置登入收到 slow_down 時從「間隔加倍」變成「加 5 秒」。→ 兩者都符合 Server 的要求；既有測試若斷言加倍，同批更新並在任務裡列出。
- [存取金鑰的驗證端點改變] 不是專案成員的人今天在 CLI 登入時就被擋下，之後是登入成功、執行動詞時才被擋。→ 寫進 D6；`auth status` 仍會顯示 repo 驗證的結果。

## 稽核結論

任務 5.1 對新增的介面套用 sharp-edges 清單（壞蛋、懶惰的開發者、搞混的開發者三個角度）。

**確認沒有問題的項目：**

- 回呼監聽只綁 `127.0.0.1` 的 0 號埠，只接受對 `/callback` 的 GET；其他路徑回 404 並繼續等。
- state 不符（包括帶 `error` 的回呼）回 400 並繼續等，回應頁面是固定文字，不含預期的 state。state 的比較不在第一個不同的位元組就停。
- verifier、授權碼、access token、refresh token 不出現在 CLI 的 stdout 與 stderr、Tauri 事件、回給瀏覽器的頁面。CLI 印出的授權網址只帶 state 與 challenge。登入服務回的錯誤只取 `error` 代碼（只留可列印的 ASCII，最多 64 個字元）。
- 發行者、授權網址、token 網址、撤銷網址都要 https；只有主機是 `127.0.0.1`、`::1`、`localhost` 時允許 http。帶 userinfo（`user@host`）的網址一律拒絕；`localhost.example.com`、`127.0.0.1.example.com` 這類主機不算本機。
- 標準文件的 `issuer` 檢查、端點的 https 檢查、S256 檢查都在任何端點被使用之前完成。對登入服務的請求不跟隨轉址。
- 換 token 成功之前，任何失敗都不寫入任何憑證；被拒絕、逾時、取消、issuer 不符、只列 plain、金鑰圈不能用都有測試確認金鑰圈沒有新增條目。
- verifier 與 state 各是 32 個來自作業系統的隨機位元組（getrandom）。
- 金鑰圈 `refresh` 條目的 JSON 解析失敗時退回裝置登入的舊格式，不會 panic。
- `pat_login` 與 `logout` 的退路參數是 `Option`，省略（`None`）時是比較安全的那一邊：只用金鑰圈、不碰憑證檔。

**找到並修正的問題（都補了測試）：**

- **Windows 的 CLI 開瀏覽器會被 `cmd.exe` 拆開網址。** 既有的做法是 `cmd /C start "" <網址>`，`cmd.exe` 把 `&` 當成指令分隔；授權網址一定含 `&`，所以瀏覽器只會收到前半段，後半段會被當成指令執行（惡意的標準文件可以藉此在使用者的電腦上執行指令）。改成 `rundll32 url.dll,FileProtocolHandler <網址>`，網址不經過任何 shell。測試確認網址原樣是最後一個參數、程式不是 `cmd`。
- **帶 token 的型別用 derive 的 `Debug`。** `RefreshEntry`、`OAuthRefresh`、`TokenResponse` 與 Desktop 的 `ConnectionLogin` 被 `{:?}` 印出時會帶出 token。改成手寫的 `Debug`，token 欄位印成 `<redacted>`，其他欄位照常可讀。
- **IPv6 主機的判斷太寬。** `http://[::1].example.com` 會被讀成主機 `::1`。改成 `]` 之後只能接 `:埠` 或結束。

**沒有改、留給之後的項目：**

- 裝置登入的驗證網址來自 Server、沒有 scheme 檢查就交給 `open`（macOS）。這是既有的行為，不在這個變更的範圍內。

## Migration Plan

沒有資料搬移。既有的金鑰圈條目與憑證檔照常可讀。要退回時還原這個變更的 commit；用新版瀏覽器登入存下的條目，舊版會在第一次換新時清掉，使用者重新登入即可。

## Open Questions

- 登入服務要求固定的回呼路徑時，`/callback` 是否足夠。目前定為固定值；有宿主提出不同的需求時再開放設定。
- 標準文件的快取：目前每次登入與每次換新都不重新取得文件（換新用的 token 網址存在條目裡）。登入服務搬移 token 網址時使用者要重新登入一次，這個代價可以接受。

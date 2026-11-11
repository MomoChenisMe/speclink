## Why

用戶端（CLI 與 Speclink Desktop）今天有兩種登入：裝置登入與存取金鑰（PAT）。兩種都假設 Server 是官方的示範 server。

- **裝置登入需要 Server 有一個核准頁。** 使用者要在瀏覽器打開 Server 的網頁、輸入裝置碼、按核准。內嵌引擎的宿主常常沒有這樣的網頁，但它有自己的登入服務（公司的單一登入）。
- **存取金鑰要使用者自己去產生再貼上。** 可用，但不是使用者期待的登入方式。
- **用戶端不知道 Server 支援什麼。** 握手回應裡有一個登入方式的欄位，但握手本身要先帶 token，而且 server 從來沒填、用戶端也從來沒讀。今天的做法是先試裝置登入，收到 404 才改問存取金鑰。
- **登入程式有兩份。** CLI 與 Speclink Desktop 共用了金鑰圈與換新 token 的程式，但登入的迴圈、存取金鑰的存放位置、登出要清什麼，兩邊各寫一份，而且不一致：CLI 把存取金鑰存在憑證檔，Desktop 存在金鑰圈，所以在一邊用存取金鑰登入後，另一邊看不到。

討論 embedded-host-needs 的裁定是：登入的伺服器端全部由宿主負責，speclink 的用戶端只實作公開標準的用戶端那一半。這個變更做三件事：新增瀏覽器登入、讓用戶端在登入前先問 Server 支援哪些方式、把登入程式收成一份。

- **目標使用者**：用 CLI 或 Speclink Desktop 連到 Server 的開發者；替團隊架設宿主的整合者（他們只要讓自己的登入服務提供標準文件）。
- **使用情境**：第一次連到一台 Server（`speclink auth login`、Desktop 的「新增專案 → Server」與「設定 → 伺服器」），以及之後 token 過期時的自動換新。對應 apply、review、verify 這些在工程師電腦上執行的階段開始之前。
- **既有規格的對照**：remote-auth 規定 CLI 的存取金鑰登入、裝置登入、四層的憑證解析與登出；desktop-connections 規定 Desktop 的裝置登入、存取金鑰的退路、登出與伺服器頁；client-protocol 規定線上型別。沒有任何正式規格涵蓋瀏覽器登入，也沒有規定登入前的探索。

Source discussion: embedded-host-needs（cut 8）。

## What Changes

- **登入方式的探索**：用戶端在登入前向 Server 送一個不帶 token 的請求，路徑是 API 根底下的 `auth-methods`。回應列出 Server 支援的方式（`browser`、`device`、`token`），支援瀏覽器登入時多帶登入服務的發行者網址、用戶端識別與選填的 scope。線上型別加進 `speclink-protocol`。Server 沒有這個路徑（回 404 或 405）時，用戶端維持今天的行為：先試裝置登入，不支援才改問存取金鑰。
- **選擇的順序**：在 Server 宣告的方式裡依固定順序選第一個——瀏覽器登入、裝置登入、存取金鑰。使用者可以指定。
- **瀏覽器登入**（RFC 8252 給桌面程式的做法，加 RFC 7636 的 PKCE）：
  1. 依發行者網址取得登入服務的標準文件（RFC 8414；找不到時改試 OpenID Connect 的探索文件）。
  2. 在 `127.0.0.1` 開一個隨機埠等待回呼。
  3. 開系統瀏覽器到授權網址，帶 PKCE 的 S256 challenge 與隨機的 state。
  4. 收到回呼後檢查 state，向 token 網址換取 access token 與 refresh token。
  5. 把 refresh token 與它需要的資訊存進 OS 金鑰圈，access token 連同到期時刻放進既有的快取。
- **token 過期時換新**：瀏覽器登入拿到的 refresh token 向登入服務的 token 網址換新，沿用既有的跨程序鎖與快取。對動詞來說和今天裝置登入的換新一樣：使用者看不到。
- **登入程式收成一份**，放在 `speclink-remote`：探索、選擇、三種登入、登出。CLI 與 Speclink Desktop 都呼叫它，只各自提供「怎麼顯示訊息、怎麼開瀏覽器、使用者有沒有取消」。Desktop 等待登入的輪詢從前端的計時器改到 Rust 這一份；前端只依階段顯示畫面。
- **存取金鑰的存放位置統一**：兩邊都存進 OS 金鑰圈。金鑰圈不能用時（例如沒有桌面環境的 CI），CLI 照舊存進使用者設定目錄的憑證檔。既有憑證檔裡的金鑰照常讀得到。
- **Server 位址的正規化統一**：兩邊用同一個函式把位址轉成金鑰圈的鍵（scheme 與主機轉小寫、只接受 http 與 https）。
- **登出統一**：清掉這個 Server 在本機的全部憑證（refresh、access token 快取、存取金鑰、憑證檔的條目）。瀏覽器登入的 refresh token，登入服務有撤銷網址時盡力撤銷。
- **登入後的身分查詢**：先問 API 根底下的 `whoami`，Server 沒有時退回既有的路徑。
- **CLI 指令**：`speclink auth login` 新增兩個旗標 `--browser` 與 `--device`，和既有的 `--pat`、`--token-stdin` 互斥。
  - 不帶旗標且在互動的終端機：依探索的結果選擇。
  - `--browser`：強制瀏覽器登入。Server 沒有宣告瀏覽器登入時以非 0 結束並說明。
  - `--device`：強制裝置登入。
  - `--pat`、`--token-stdin`：行為與輸出不變；stdin 的用法不變。
  - 不帶旗標且不是互動的終端機：照舊以非 0 結束並指引 `--token-stdin`。
  - exit code：成功 0；被拒絕、逾時、取消、Server 不支援指定的方式、金鑰圈不能用都是非 0，訊息可以分辨。
  - 瀏覽器登入時 stdout 印出授權網址（瀏覽器沒有自動開啟時使用者可以自己貼上）。
- **Speclink Desktop**：「新增專案 → Server」與「設定 → 伺服器」的登入多一個「等待瀏覽器登入」的階段，就地顯示說明、可複製的網址、倒數與取消。「登入方式」卡改成三列。
- **相容性影響**：
  - 連到今天的官方 server：它沒有 `auth-methods`，所以登入流程、輸出與 exit code 都不變。
  - `speclink auth login --pat` 與 `--token-stdin`：輸出不變。存放位置在有金鑰圈的機器上從憑證檔改成金鑰圈，`speclink auth status --json` 的 `credentialSource` 因此從 `credentials_file` 變成 `keychain_pat`。既有憑證檔的條目不搬移、照常可用；`auth logout` 會清掉它。
  - `auth status --json` 的欄位與值域不變：瀏覽器登入的憑證來源顯示為既有的 `keychain_refresh`。
  - Server 位址含大寫主機名稱的既有登入：正規化後金鑰圈的鍵不同，要重新登入一次。
  - 同一台電腦上新舊版本混用：舊版讀到瀏覽器登入存的條目時會當成裝置登入的 refresh token 去換新，被拒絕後清掉它。只有宿主啟用瀏覽器登入時才會遇到，而舊版本來就不能用瀏覽器登入。
- **設定欄位**：`speclink/config.yaml` 與 `.speclink.yaml` 沒有新增或變更。
- **影響的技能**：沒有。

## Non-Goals

- **登入的伺服器端**：授權頁、發 token、驗 token、帳號。這些是宿主的登入服務的事，speclink 不定義也不實作。
- **Server 回答 `auth-methods` 與 API 根的 `whoami`**：由變更 host-kit-client-door 在宿主套件的入口做。這個變更只做用戶端與線上型別；測試用假的 Server 與假的登入服務。
- **官方示範 server 支援瀏覽器登入**：它沒有 OAuth 的授權端點，維持裝置登入與存取金鑰。
- **裝置登入改成 RFC 8628 的標準格式**：維持今天 speclink 自己的格式。
- **自訂 URL scheme 的回呼**（`speclink://`）：RFC 8252 允許，但本機埠在三個平台都不用註冊。
- **把既有憑證檔的金鑰搬進金鑰圈**：不搬，只改之後的寫入位置。
- **Desktop 讀取環境變數 `SPECLINK_TOKEN` 或憑證檔**：維持 Desktop 的憑證只在金鑰圈。
- **使用者文件**：這個變更不動 README 與 docs。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `remote-auth`: 新增「登入方式的探索與選擇」與「瀏覽器登入」；修改「PAT 登入與憑證儲存」（存放位置）、「裝置授權登入」（不帶旗標時改由選擇決定、新增 `--device`）、「登出」（瀏覽器登入的撤銷）。
- `desktop-connections`: 修改「credential 唯一落點為 OS Keychain」（列入瀏覽器登入的憑證）、「device login 預設與 PAT fallback」（先探索再選擇、多一個等待瀏覽器登入的階段）、「登出撤銷與移除連帶清理」（清掉的範圍與 CLI 相同）、「伺服器管理最小面」（登入方式卡三列）。
- `client-protocol`: 新增「登入方式宣告的 payload」。

## Impact

- Affected specs: remote-auth（新增兩條、修改三條）、desktop-connections（修改四條）、client-protocol（新增一條）
- Affected crates 與 app: `speclink-protocol`、`speclink-remote`、`speclink-cli`、apps/desktop（Tauri 殼與前端）。`speclink-core`、`speclink-host`、`speclink-server`、store crates 不改。
- Affected code:
  - New:
    - crates/protocol/speclink-protocol/src/auth.rs
    - crates/protocol/speclink-remote/src/discovery.rs
    - crates/protocol/speclink-remote/src/oauth.rs
    - crates/protocol/speclink-remote/src/loopback.rs
    - crates/protocol/speclink-remote/tests/it/browser_login.rs
    - crates/protocol/speclink-remote/tests/it/login_methods.rs
    - crates/adapters/speclink-cli/tests/it/auth_login_methods.rs
  - Modified:
    - crates/protocol/speclink-protocol/src/lib.rs
    - crates/protocol/speclink-remote/Cargo.toml
    - crates/protocol/speclink-remote/src/lib.rs
    - crates/protocol/speclink-remote/src/login.rs
    - crates/protocol/speclink-remote/src/auth.rs
    - crates/protocol/speclink-remote/src/refresh.rs
    - crates/protocol/speclink-remote/src/credentials.rs
    - crates/protocol/speclink-remote/src/device.rs
    - crates/protocol/speclink-remote/tests/it/main.rs
    - crates/adapters/speclink-cli/src/verbs/connection.rs
    - crates/adapters/speclink-cli/tests/it/main.rs
    - apps/desktop/src-tauri/src/connections.rs
    - apps/desktop/src-tauri/src/lib.rs
    - apps/desktop/src-tauri/src/remote.rs
    - apps/desktop/src/store.ts
    - apps/desktop/src/adapter/connections.ts
    - apps/desktop/src/components/connectionLogin.tsx
    - apps/desktop/src/components/WorkspaceChooser.tsx
    - apps/desktop/src/components/ServersPanel.tsx
    - apps/desktop/src/views/settings/ServersPage.tsx
    - apps/desktop/src/i18n/messages.ts
    - apps/desktop/src/__tests__/serversPage.test.tsx
  - Removed: (none)
- 新的相依套件（都已經在 Cargo.lock 裡，由其他 crate 帶入）：`speclink-remote` 直接相依 sha2、base64、getrandom。
- 相依：沒有。可以和 host-kit-client-door 並行；兩邊靠 `speclink-protocol` 的線上型別對齊，型別由這個變更加入。
- 三種形態下的結果：
  - 本機 repo：不適用。本機專案不登入。
  - 連到遠端的 checkout：可用。連到官方示範 server 時行為不變；連到宣告瀏覽器登入的宿主時用瀏覽器登入。已知缺口：在 host-kit-client-door 完成之前，沒有任何 Server 會回答 `auth-methods`，所以瀏覽器登入只能對測試用的假 Server 驗證。
  - 沒有本機規格檔的內嵌宿主：不適用。宿主自己的 Agent 在宿主的程序裡，身分由宿主的程式給，不經過用戶端的登入。
- 文件影響（留給最後一個變更 docs-embedded-host，這個變更不動使用者文件）：
  - Remote 入門與 CLI 的動詞與旗標說明：`auth login` 的新旗標、三種登入方式與選擇順序、存取金鑰的存放位置。
  - 「把 Speclink 內嵌進你的系統」：宿主要支援瀏覽器登入時，登入服務要提供標準文件、接受 `http://127.0.0.1` 任意埠的回呼、把 speclink 的用戶端登記成公開用戶端（不帶密鑰）。
  - Speclink Desktop 的說明與截圖：伺服器頁的登入方式卡。

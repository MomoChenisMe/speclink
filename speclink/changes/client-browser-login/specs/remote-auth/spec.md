## ADDED Requirements

### Requirement: 登入方式的探索與選擇

speclink auth login 不帶 --pat 與 --token-stdin 時 SHALL 在登入前向 server 送出一個不帶憑證的探索請求（API 根底下的 auth-methods），取得 server 宣告支援的登入方式：browser（瀏覽器登入）、device（裝置授權）、token（存取金鑰）。不認得的方式名稱 SHALL 被略過。沒有指定旗標時 SHALL 在宣告的方式裡依固定順序選第一個：browser、device、token；這個順序 SHALL NOT 隨回應裡的排列改變。選到 token 時 SHALL 以非 0 exit code 結束並指引 --pat 或 --token-stdin。server 對探索回應 404 或 405 時視為沒有宣告，SHALL 維持既有行為：走裝置授權，server 不支援裝置授權時指引 --pat。探索回應其他錯誤狀態或 server 不可達時 SHALL 以非 0 exit code 報連線錯誤，SHALL NOT 改用任何登入方式。帶 --browser 或 --device 而 server 有宣告但不含該方式時 SHALL 以非 0 exit code 結束，stderr SHALL 列出 server 支援的方式。帶 --browser 而 server 沒有宣告時 SHALL 以非 0 exit code 結束並說明這台 server 沒有宣告瀏覽器登入。宣告含 browser 但缺少發行者網址或用戶端識別時 SHALL 視為沒有宣告 browser。

#### Scenario: 宣告瀏覽器登入與裝置授權時選瀏覽器登入

- **WHEN** server 宣告 methods 為 device、browser（browser 帶發行者網址與用戶端識別），於互動 TTY 執行 speclink auth login
- **THEN** 走瀏覽器登入：stdout 印出登入服務的授權網址，沒有印出 user code

#### Scenario: 只宣告存取金鑰

- **WHEN** server 宣告 methods 只有 token，於互動 TTY 執行 speclink auth login
- **THEN** exit code 非 0，stderr 指引 --pat 或 --token-stdin；沒有發起裝置授權

#### Scenario: 沒有宣告時維持既有行為

- **WHEN** server 對探索回應 404，於互動 TTY 執行 speclink auth login
- **THEN** 走裝置授權，流程與輸出和既有行為相同

#### Scenario: 探索失敗不猜

- **WHEN** server 對探索回應 500，於互動 TTY 執行 speclink auth login
- **THEN** exit code 非 0，stderr 說明連線錯誤；沒有發起任何登入流程

#### Scenario: 指定的方式不被支援

- **WHEN** server 宣告 methods 只有 device，執行 speclink auth login --browser
- **THEN** exit code 非 0，stderr 說明這台 server 不支援瀏覽器登入並列出 device；沒有開啟瀏覽器

#### Scenario: 對沒有宣告的 server 指定瀏覽器登入

- **WHEN** server 對探索回應 404，執行 speclink auth login --browser
- **THEN** exit code 非 0，stderr 說明這台 server 沒有宣告瀏覽器登入

#### Scenario: 不認得的方式被略過

- **WHEN** server 宣告 methods 為 passkey、device，於互動 TTY 執行 speclink auth login
- **THEN** 走裝置授權，沒有因為 passkey 而報錯

#### Scenario: 非互動且無旗標不探索

- **WHEN** stdin 非 TTY 且不帶任何旗標執行 speclink auth login
- **THEN** exit code 非 0，stderr 指引使用 --token-stdin，不發起任何網路請求

### Requirement: 瀏覽器登入

選到瀏覽器登入時，CLI SHALL 依 server 宣告的發行者網址取得登入服務的標準文件（RFC 8414 的位置；該位置回應 404 時改用 OpenID Connect 的探索文件），並 SHALL 在使用前確認：文件的 issuer 與宣告的發行者網址完全相同、含授權端點與 token 端點、列有支援的 PKCE 方式時含 S256；任一項不符 SHALL 以非 0 exit code 結束並指出是哪一項。發行者、授權端點與 token 端點 SHALL 是 https，主機為本機位址時例外。CLI SHALL 在 127.0.0.1 的一個由作業系統指定的埠等待回呼，回呼位址 SHALL 是 `http://127.0.0.1:<埠>/callback`，SHALL NOT 使用 localhost。授權請求 SHALL 帶 PKCE 的 S256 challenge 與一個隨機的 state；CLI SHALL 於 stdout 印出授權網址，並在可開啟瀏覽器的環境同時開啟它。回呼的 state 與送出的不同時 SHALL 不接受該回呼並繼續等待。收到有效的回呼後 SHALL 以授權碼與 PKCE verifier 向 token 端點換取 token，SHALL NOT 送出任何用戶端密鑰。成功後 SHALL 將 refresh credential 與短效 access token（連同到期時刻）存入 OS 金鑰圈、清除該 origin 的金鑰圈 PAT 條目、顯示登入身分資訊、exit code 0，SHALL NOT 寫入憑證檔；speclink auth status 的 credentialSource SHALL 是 keychain_refresh。使用者在登入服務拒絕、300 秒內沒有回呼、token 端點拒絕，SHALL 各以非 0 exit code 結束且訊息可區分，金鑰圈 SHALL 無新增條目。OS 金鑰圈不可用時 SHALL 在開啟瀏覽器之前以非 0 exit code 結束並指引 --pat 或 SPECLINK_TOKEN。之後 access token 到期時 SHALL 以 refresh credential 向登入服務的 token 端點換新，換新的序列化、快取與「未授權時重試恰一次」的規則 SHALL 與裝置授權的 refresh credential 相同；登入服務拒絕換新時 SHALL 清除該 origin 的金鑰圈 refresh 與 access token 快取條目並提示 speclink auth login。回應瀏覽器的頁面 SHALL NOT 含授權碼或任何 token。

#### Scenario: 瀏覽器登入完整流程

- **WHEN** server 宣告瀏覽器登入，於互動 TTY 執行 speclink auth login，使用者在瀏覽器完成登入
- **THEN** stdout 曾印出授權網址，網址含 code_challenge_method=S256、state 與指向 127.0.0.1 某個埠的 redirect_uri；完成後顯示身分資訊、exit code 0；金鑰圈存有該 origin 的 refresh credential 與 access token 條目，憑證檔無新增內容

#### Scenario: 使用者在登入服務拒絕

- **WHEN** 瀏覽器登入等待回呼時，登入服務以 error=access_denied 回呼
- **THEN** exit code 非 0，stderr 說明被拒絕（與逾時訊息可區分），金鑰圈無新增條目

#### Scenario: 逾時

- **WHEN** 瀏覽器登入開始後 300 秒內沒有收到有效的回呼
- **THEN** exit code 非 0，stderr 說明逾時，金鑰圈無新增條目，本機的埠已關閉

#### Scenario: state 不符的回呼不被接受

- **WHEN** 等待回呼時先收到一個 state 錯誤的回呼，之後收到 state 正確的回呼
- **THEN** 第一個回呼得到錯誤回應且登入沒有結束；第二個回呼完成登入

#### Scenario: 標準文件的 issuer 不符

- **WHEN** 登入服務的標準文件裡 issuer 與 server 宣告的發行者網址不同
- **THEN** exit code 非 0，stderr 指出 issuer 不符；沒有開啟瀏覽器

#### Scenario: 只有 OpenID Connect 探索文件的登入服務

- **WHEN** 登入服務對 RFC 8414 的位置回應 404，但提供 OpenID Connect 的探索文件
- **THEN** 瀏覽器登入照常完成

#### Scenario: 不支援 S256 的登入服務

- **WHEN** 標準文件列出的 PKCE 方式只有 plain
- **THEN** exit code 非 0，stderr 指出登入服務不支援 S256；沒有開啟瀏覽器

#### Scenario: 發行者不是 https

- **WHEN** server 宣告的發行者網址是 http 且主機不是本機位址
- **THEN** exit code 非 0，stderr 指出發行者必須是 https

#### Scenario: 金鑰圈不可用時拒絕瀏覽器登入

- **WHEN** 平台無金鑰圈服務，server 宣告瀏覽器登入，於互動 TTY 執行 speclink auth login
- **THEN** exit code 非 0，stderr 說明無法安全儲存 refresh credential 並指引 --pat 或 SPECLINK_TOKEN；沒有開啟瀏覽器

#### Scenario: access token 到期的靜默換新

- **WHEN** 以瀏覽器登入後 access token 快取已到期但 refresh credential 有效，執行 speclink list --json
- **THEN** 指令成功輸出，過程無任何登入提示；登入服務的 token 端點收到一次 refresh_token 的請求；金鑰圈的條目已更新

#### Scenario: 併發換新僅一次

- **WHEN** 以瀏覽器登入後 access token 快取已到期，兩個行程同時執行 remote 動詞
- **THEN** 登入服務的 token 端點僅收到一次換新請求，兩個動詞皆成功

#### Scenario: 登入服務拒絕換新

- **WHEN** 以瀏覽器登入後 refresh credential 已被登入服務撤銷，執行 speclink list
- **THEN** exit code 非 0 並提示 speclink auth login；金鑰圈該 origin 的 refresh 與 access token 條目被清除

#### Scenario: 登入服務沒有發 refresh token

- **WHEN** token 端點的回應沒有 refresh token，登入完成後等到 access token 到期，執行 speclink list
- **THEN** 登入當下成功；到期後的指令以非 0 exit code 結束並提示 speclink auth login

## MODIFIED Requirements

### Requirement: PAT 登入與憑證儲存
<!-- BEFORE: PAT 一律存入使用者層級設定目錄的憑證檔；只有 --pat 與 --token-stdin 兩個旗標互斥 -->

speclink auth login --pat SHALL 互動接受 PAT 輸入；speclink auth login --token-stdin SHALL 自 stdin 讀取 PAT（CI／腳本用），其行為、輸出與 exit code SHALL 維持既有位元級不變。兩者 SHALL 先以身分查詢確認 PAT 有效，再依連接 url 正規化後的 origin（scheme 與主機轉小寫）將 PAT 存入 OS 金鑰圈；金鑰圈不可用（無服務或存取被拒）時 SHALL 改存入使用者層級設定目錄的憑證檔（Unix 檔案權限 0600）。兩者 SHALL NOT 將憑證寫入專案 repo 內的任何檔案，SHALL NOT 先探索 server 宣告的登入方式。PAT 登入成功時 SHALL 清除該 origin 的金鑰圈 refresh 與 access token 快取條目。既有憑證檔中的 PAT SHALL 照常可讀，SHALL NOT 被搬移。--browser、--device、--pat、--token-stdin 四個旗標中任兩個同時給定 SHALL 以非 0 exit code 拒絕。環境變數 SPECLINK_TOKEN 存在時 SHALL 優先於所有其他憑證來源。speclink auth status SHALL 查驗當前解析所得憑證並顯示身分與 repo 驗證結果，且 SHALL 標示憑證來源層：人眼輸出增列來源描述，--json 新增 credentialSource 欄位（string，值域 env、keychain_refresh、keychain_pat、credentials_file），既有欄位不變。

#### Scenario: --pat 登入後憑證落於使用者目錄
- **WHEN** 於 remote 模式專案執行 speclink auth login --pat 並提供有效 PAT
- **THEN** 憑證寫入 OS 金鑰圈的 PAT 條目（專案 repo 內無任何新增或變更的檔案，憑證檔無新增內容），指令顯示登入成功與身分資訊，exit code 0；speclink auth status --json 的 credentialSource 為 keychain_pat

#### Scenario: SPECLINK_TOKEN 覆寫憑證檔
- **WHEN** 憑證檔含某 token A，環境變數 SPECLINK_TOKEN 設為 token B，執行 speclink auth status
- **THEN** 查驗以 token B 進行，--json 的 credentialSource 為 env

#### Scenario: 未登入的狀態查詢
- **WHEN** 無環境變數、金鑰圈無任何條目、無憑證檔，執行 speclink auth status
- **THEN** 顯示未登入狀態與 speclink auth login 指引，exit code 非 0

#### Scenario: auth status 標示金鑰圈來源
- **WHEN** 金鑰圈存有有效 refresh credential（desktop 登入所建），無環境變數，執行 speclink auth status --json
- **THEN** payload 含 credentialSource 欄位且值為 keychain_refresh，身分資訊照常顯示

#### Scenario: 旗標互斥
- **WHEN** 執行 speclink auth login --pat --token-stdin
- **THEN** exit code 非 0，stderr 說明兩旗標互斥，不進入任何登入流程

#### Scenario: 金鑰圈不可用時 PAT 存入憑證檔

- **WHEN** 平台無金鑰圈服務，執行 speclink auth login --token-stdin 並由 stdin 提供有效 PAT
- **THEN** 憑證寫入使用者層級設定目錄的憑證檔（Unix 檔案權限 0600），exit code 0；speclink auth status --json 的 credentialSource 為 credentials_file

#### Scenario: CLI 以 PAT 登入後 desktop 視為已登入

- **WHEN** 於有金鑰圈的機器以 speclink auth login --pat 對某 origin 登入成功，之後開啟 desktop 的伺服器頁
- **THEN** 該 origin 的連線列顯示已登入

#### Scenario: 既有憑證檔的 PAT 照常可用

- **WHEN** 憑證檔存有升級前寫入的有效 PAT，金鑰圈無任何條目，執行 speclink list
- **THEN** 指令以憑證檔 PAT 成功；憑證檔內容不變，金鑰圈無新增條目

#### Scenario: 無效的 PAT 不留憑證

- **WHEN** 執行 speclink auth login --token-stdin 並提供 server 不接受的 PAT
- **THEN** exit code 非 0，stderr 說明認證失敗；金鑰圈與憑證檔皆無新增內容

### Requirement: 裝置授權登入
<!-- BEFORE: 互動 TTY 且無旗標時一律走裝置授權；沒有 --device 旗標 -->

speclink auth login 帶 --device 時，或於互動 TTY 且無旗標而依「登入方式的探索與選擇」選到裝置授權時，SHALL 走裝置授權：向 server 發起授權，stdout SHALL 印出 verification URL 與 user code（供任一裝置核准），可開啟瀏覽器的環境 SHALL 同時開啟核准頁；之後依 server 宣告的最小間隔輪詢。核准後 SHALL 將 refresh credential 與短效 access token（連同到期時刻）存入 OS 金鑰圈、顯示登入身分資訊、exit code 0，SHALL NOT 寫入憑證檔，並 SHALL 清除該 origin 的金鑰圈 PAT 條目。被拒絕與逾期 SHALL 以非 0 exit code 結束且訊息可區分兩者。server 不支援裝置授權 SHALL 以非 0 exit code 結束並指引 --pat。非互動（無 TTY）且無旗標 SHALL 以非 0 exit code 結束並指引 --token-stdin。OS 金鑰圈不可用（無服務或存取被拒）時 SHALL 以非 0 exit code 結束、說明 refresh credential 不落明文檔並指引 --pat 或 SPECLINK_TOKEN。

#### Scenario: 互動裝置授權完整流程
- **WHEN** 對沒有宣告登入方式的 server，於互動 TTY 執行 speclink auth login，於核准頁完成核准
- **THEN** stdout 曾印出 verification URL 與 user code，核准後顯示身分資訊、exit code 0；金鑰圈存有該 origin 的 refresh credential 與 access token 條目，憑證檔無新增內容

#### Scenario: 核准被拒
- **WHEN** 於互動 TTY 執行 speclink auth login，使用者於核准頁拒絕
- **THEN** exit code 非 0，stderr 訊息說明被拒絕（與逾期訊息可區分），金鑰圈無新增條目

#### Scenario: 非互動且無旗標
- **WHEN** stdin 非 TTY 且不帶任何旗標執行 speclink auth login
- **THEN** exit code 非 0，stderr 指引使用 --token-stdin，不發起任何網路請求

#### Scenario: 金鑰圈不可用時拒絕裝置授權
- **WHEN** 平台無金鑰圈服務，於互動 TTY 執行 speclink auth login
- **THEN** exit code 非 0，stderr 說明無法安全儲存 refresh credential 並指引 --pat 或 SPECLINK_TOKEN，未發起裝置授權

#### Scenario: --device 指定裝置授權

- **WHEN** server 宣告支援瀏覽器登入與裝置授權，於互動 TTY 執行 speclink auth login --device
- **THEN** stdout 印出 verification URL 與 user code，沒有開啟登入服務的授權網址；核准後 exit code 0

#### Scenario: 對沒有宣告的 server 輸出不變

- **WHEN** server 對登入方式的探索回應 404，於互動 TTY 執行 speclink auth login 並完成核准
- **THEN** stdout、stderr 與 exit code 和這個需求修改前逐位元相同

### Requirement: 登出
<!-- BEFORE: 只撤銷裝置授權的 credential family；瀏覽器登入不存在 -->

speclink auth logout：金鑰圈存有裝置授權的 refresh credential 時 SHALL 呼叫 server 撤銷其 credential family；存有瀏覽器登入的 refresh credential 且登入時取得的標準文件列有撤銷端點時 SHALL 向該端點送出撤銷請求，之後 SHALL 清除該 origin 的所有本機憑證——金鑰圈的 refresh、access token 快取與 PAT 條目，及憑證檔中該 origin 的條目；server 端的 PAT SHALL NOT 被撤銷。成功 SHALL exit code 0 並顯示已登出的 origin。該 origin 全無本機憑證時 SHALL 以非 0 exit code 報未登入。撤銷請求網路失敗時 SHALL 仍清除本機憑證、exit code 0，並於 stderr 警告 server 端 family 未撤銷。

#### Scenario: 登出撤銷 family 並清除本機憑證
- **WHEN** 金鑰圈存有 refresh credential 與 access token 快取、憑證檔存有同 origin PAT，執行 speclink auth logout
- **THEN** server 收到撤銷請求，金鑰圈該 origin 條目與憑證檔該 origin 條目皆被清除，exit code 0；desktop 對同 origin 的下一次操作回到未登入狀態

#### Scenario: 未登入時登出
- **WHEN** 該 origin 無任何本機憑證，執行 speclink auth logout
- **THEN** exit code 非 0，stderr 顯示未登入

#### Scenario: 撤銷請求網路失敗
- **WHEN** 金鑰圈存有 refresh credential 但 server 不可達，執行 speclink auth logout
- **THEN** 本機憑證仍被清除，exit code 0，stderr 警告 server 端 credential family 未被撤銷

#### Scenario: 登出瀏覽器登入的憑證

- **WHEN** 金鑰圈存有瀏覽器登入的 refresh credential，登入服務的標準文件列有撤銷端點，執行 speclink auth logout
- **THEN** 登入服務的撤銷端點收到該 refresh token 的撤銷請求；金鑰圈該 origin 的 refresh 與 access token 快取條目被清除；exit code 0

#### Scenario: 登入服務沒有撤銷端點

- **WHEN** 金鑰圈存有瀏覽器登入的 refresh credential，登入服務的標準文件沒有撤銷端點，執行 speclink auth logout
- **THEN** 不送出任何撤銷請求；本機憑證被清除；exit code 0

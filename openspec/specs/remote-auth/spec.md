# remote-auth Specification

## Purpose

CLI 對 remote server 的認證：PAT 登入與裝置授權登入兩條路徑、憑證的儲存與四層解析階梯（SPECLINK_TOKEN 環境變數 → 金鑰圈 refresh credential → 金鑰圈 PAT → 憑證檔 PAT）、憑證失效時的處理，以及登出。本 capability 保證某一層不可用時靜默續探下一層而非讓動詞失敗，且同機共用 credential family 的多個程序在換發 token 時序列化進行、不會互相把對方的憑證換掉。

## Requirements

### Requirement: PAT 登入與憑證儲存
speclink auth login --pat SHALL 互動接受 PAT 輸入；speclink auth login --token-stdin SHALL 自 stdin 讀取 PAT（CI／腳本用），其行為、輸出與 exit code SHALL 維持既有位元級不變。兩者 SHALL 依連接 url 的 origin 將 PAT 存入使用者層級設定目錄的憑證檔（Unix 檔案權限 0600），SHALL NOT 將憑證寫入專案 repo 內的任何檔案。--pat 與 --token-stdin 同時給定 SHALL 以非 0 exit code 拒絕。環境變數 SPECLINK_TOKEN 存在時 SHALL 優先於所有其他憑證來源。speclink auth status SHALL 查驗當前解析所得憑證並顯示身分與 repo 驗證結果，且 SHALL 標示憑證來源層：人眼輸出增列來源描述，--json 新增 credentialSource 欄位（string，值域 env、keychain_refresh、keychain_pat、credentials_file），既有欄位不變。

#### Scenario: --pat 登入後憑證落於使用者目錄
- **WHEN** 於 remote 模式專案執行 speclink auth login --pat 並提供有效 PAT
- **THEN** 憑證寫入使用者層級設定目錄的憑證檔（專案 repo 內無任何新增或變更的檔案），指令顯示登入成功與身分資訊，exit code 0

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


<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->

---
### Requirement: 憑證失效的處理
remote 動詞收到未授權回應時：憑證來源為金鑰圈 refresh 者，CLI SHALL 以同一 credential family 換發重試恰一次，換發成功即以新憑證完成該動詞、無任何使用者可見的登入提示；換發被 server 拒絕（family 已撤銷）SHALL 清除該 origin 的金鑰圈 refresh 與 access token 快取條目、以非 0 exit code 結束並提示 speclink auth login。憑證來源為其他層（環境變數、PAT）時 SHALL 維持既有行為：非 0 exit code、提示重新登入、不重試。單次動詞執行內 SHALL NOT 靜默改用其他憑證來源。

#### Scenario: token 撤銷後的動詞行為
- **WHEN** 憑證來源為 PAT 且已被 server 撤銷，執行 speclink list
- **THEN** exit code 非 0，stderr 單行訊息說明認證失效並提示 speclink auth login，指令不重試

#### Scenario: access token 到期的靜默換發
- **WHEN** 金鑰圈 access token 快取已到期但 refresh credential 有效，執行 speclink list --json
- **THEN** 指令成功輸出，過程無任何登入提示，金鑰圈 refresh 與 access token 條目已更新為新值

#### Scenario: refresh family 撤銷後的動詞行為
- **WHEN** 憑證來源為金鑰圈 refresh 且其 family 已被 server 撤銷，憑證檔另存有效 PAT，執行 speclink list
- **THEN** exit code 非 0 並提示 speclink auth login，金鑰圈 refresh 與 access token 條目被清除，該次執行不改用憑證檔 PAT；再次執行 speclink list 時以憑證檔 PAT 成功


<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->

---
### Requirement: 裝置授權登入
speclink auth login 於互動 TTY 且無旗標時 SHALL 走裝置授權：向 server 發起授權，stdout SHALL 印出 verification URL 與 user code（供任一裝置核准），可開啟瀏覽器的環境 SHALL 同時開啟核准頁；之後依 server 宣告的最小間隔輪詢。核准後 SHALL 將 refresh credential 與短效 access token（連同到期時刻）存入 OS 金鑰圈、顯示登入身分資訊、exit code 0，SHALL NOT 寫入憑證檔。被拒絕與逾期 SHALL 以非 0 exit code 結束且訊息可區分兩者。server 不支援裝置授權 SHALL 以非 0 exit code 結束並指引 --pat。非互動（無 TTY）且無旗標 SHALL 以非 0 exit code 結束並指引 --token-stdin。OS 金鑰圈不可用（無服務或存取被拒）時 SHALL 以非 0 exit code 結束、說明 refresh credential 不落明文檔並指引 --pat 或 SPECLINK_TOKEN。

#### Scenario: 互動裝置授權完整流程
- **WHEN** 於互動 TTY 執行 speclink auth login，於核准頁完成核准
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

<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->

---
### Requirement: 憑證解析階梯
remote 動詞與 auth status SHALL 依固定順序解析憑證：SPECLINK_TOKEN 環境變數 → 金鑰圈 refresh credential（經 access token 快取與換發）→ 金鑰圈 PAT → 憑證檔 PAT。某層不可用（平台無金鑰圈服務、金鑰圈存取被拒、條目不存在）SHALL 靜默續探下一層，SHALL NOT 因金鑰圈不可用而使動詞失敗。四層皆無憑證 SHALL 以非 0 exit code 報未登入並指引 speclink auth login。

#### Scenario: desktop 登入後 CLI 免登入
- **WHEN** desktop 已對某 origin 完成裝置授權登入（金鑰圈存有 refresh credential），同機以無環境變數、無憑證檔的狀態執行 speclink list --json
- **THEN** 指令成功輸出，全程無任何登入提示

##### Example: 共享後的首個動詞
- **GIVEN** desktop 剛完成對 http://localhost:8080 的裝置授權登入，CLI 從未執行過 auth login
- **WHEN** 執行 speclink workflow-config show --json
- **THEN** 指令輸出 workflow config 的 JSON，exit code 0，stderr 無 Not logged in 訊息

#### Scenario: 無金鑰圈平台回退憑證檔
- **WHEN** 平台無金鑰圈服務（headless CI），憑證檔存有有效 PAT，執行 speclink list
- **THEN** 指令以憑證檔 PAT 成功，stderr 無任何金鑰圈相關錯誤

#### Scenario: 環境變數優先於金鑰圈
- **WHEN** SPECLINK_TOKEN 設為有效 token，金鑰圈同時存有另一身分的 refresh credential，執行 speclink auth status
- **THEN** 顯示環境變數 token 的身分，credentialSource 為 env

<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->

---
### Requirement: 共用 credential family 與換發序列化
同機同 origin 的 desktop 與 CLI SHALL 讀寫同一金鑰圈 refresh 條目（單一 credential family）。refresh 換發與裝置授權登入的 credential 寫入 SHALL 以使用者設定目錄下的獨立鎖檔跨行程序列化：併發的換發需求 SHALL 恰產生一次 server 端換發，SHALL NOT 觸發 server 的 reuse 偵測（整族撤銷）。短效 access token SHALL 連同到期時刻快取於金鑰圈；未到期時 SHALL 直接使用而不發起換發請求。取得鎖後 SHALL 重讀 access token 快取，先行者已換新時 SHALL 複用其結果而非再次換發。等待鎖 SHALL 有時間上限，逾時 SHALL 以錯誤結束並指出疑似有其他行程長時間持鎖，SHALL NOT 無限期阻塞。

#### Scenario: 併發換發僅一次
- **WHEN** access token 快取已到期，兩個行程同時執行 remote 動詞
- **THEN** server 僅收到一次換發請求，兩個動詞皆成功，無任何一方收到認證失效錯誤

#### Scenario: 等待鎖逾時不無限阻塞
- **WHEN** 鎖被另一行程長時間持有，本行程需要換發
- **THEN** 於時間上限內以錯誤結束，訊息指出疑似有其他 speclink 行程長時間持鎖，指令不無限期停住

#### Scenario: 快取未到期不換發
- **WHEN** access token 快取未到期，連續執行多個 remote 動詞
- **THEN** server 未收到任何換發請求，各動詞皆成功

#### Scenario: desktop 與 CLI 交錯使用不互相登出
- **WHEN** desktop 與 CLI 對同 origin 交錯執行多輪需認證的操作（各自觸發過換發）
- **THEN** 兩端全程無認證失效錯誤，金鑰圈始終存有可用的 refresh credential

<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->

---
### Requirement: 登出
speclink auth logout：金鑰圈存有 refresh credential 時 SHALL 呼叫 server 撤銷其 credential family，之後 SHALL 清除該 origin 的所有本機憑證——金鑰圈的 refresh、access token 快取與 PAT 條目，及憑證檔中該 origin 的條目；server 端的 PAT SHALL NOT 被撤銷。成功 SHALL exit code 0 並顯示已登出的 origin。該 origin 全無本機憑證時 SHALL 以非 0 exit code 報未登入。撤銷請求網路失敗時 SHALL 仍清除本機憑證、exit code 0，並於 stderr 警告 server 端 family 未撤銷。

#### Scenario: 登出撤銷 family 並清除本機憑證
- **WHEN** 金鑰圈存有 refresh credential 與 access token 快取、憑證檔存有同 origin PAT，執行 speclink auth logout
- **THEN** server 收到撤銷請求，金鑰圈該 origin 條目與憑證檔該 origin 條目皆被清除，exit code 0；desktop 對同 origin 的下一次操作回到未登入狀態

#### Scenario: 未登入時登出
- **WHEN** 該 origin 無任何本機憑證，執行 speclink auth logout
- **THEN** exit code 非 0，stderr 顯示未登入

#### Scenario: 撤銷請求網路失敗
- **WHEN** 金鑰圈存有 refresh credential 但 server 不可達，執行 speclink auth logout
- **THEN** 本機憑證仍被清除，exit code 0，stderr 警告 server 端 credential family 未被撤銷

<!-- @trace
source: cli-desktop-credential-sharing
updated: 2026-07-29
code:
  - Cargo.lock
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/connections.rs
  - apps/desktop/src-tauri/src/credentials.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src-tauri/tests/common/mod.rs
  - apps/desktop/src-tauri/tests/connections.rs
  - apps/desktop/src-tauri/tests/event_manager.rs
  - apps/desktop/src-tauri/tests/login_orchestration.rs
  - apps/desktop/src-tauri/tests/migration.rs
  - apps/desktop/src-tauri/tests/phase3_chain.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src-tauri/tests/remote_runtime.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/credentials.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/src/login.rs
  - crates/speclink-remote/src/refresh.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/credential_ladder.rs
  - crates/speclink-remote/tests/device_login_flow.rs
  - crates/speclink-remote/tests/reauth_retry.rs
  - crates/speclink-remote/tests/refresh_lock.rs
-->
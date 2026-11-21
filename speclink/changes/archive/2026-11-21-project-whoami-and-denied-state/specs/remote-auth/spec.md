## ADDED Requirements

### Requirement: auth status 的憑證來源說明與多個 repo 的專案

`speclink auth status`（不帶 `--json`）SHALL 於 stdout 依序輸出身分一行與憑證來源一行；憑證來源一行的格式 SHALL 為兩個空白加 `Credential from: <來源說明>`。憑證來自金鑰圈的登入憑證（`--json` 的 credentialSource 為 keychain_refresh）時，來源說明 SHALL 為 `system keychain (login session, shared with the desktop app)`，SHALL NOT 指名登入方式——金鑰圈的條目不記錄它來自瀏覽器登入或裝置授權登入。這是刻意的變更：變更前這一行為 `system keychain (device login, shared with the desktop app)`。其他三種來源（環境變數、金鑰圈 PAT、憑證檔）的來源說明 SHALL 維持既有的位元級輸出。`--no-color` SHALL NOT 改變這兩行的文字。

專案註冊多個 repo 時，auth status SHALL 照常完成：remote section 有 repo 時 SHALL 輸出該 repo 的查驗結果（已註冊，或以 `! ` 開頭指出未註冊並列出可用註冊名的一行）；remote section 沒有 repo 時 SHALL 輸出一行以 `! ` 開頭、指引以 `speclink link <url> --repo <key>` 指名並列出全部可用註冊名的提示。兩種情況 exit code SHALL 為 0。

`speclink auth status --json` 的 payload SHALL 不變：`user.name`（string）、`user.handle`（string）、`credentialSource`（string，值為 env、keychain_refresh、keychain_pat、credentials_file 之一）；`--json` 下 SHALL NOT 輸出 repo 的查驗結果或提示。未登入、憑證被拒與不是專案成員時的訊息與非 0 exit code SHALL 維持既有行為。

#### Scenario: 瀏覽器登入後查看狀態

- **WHEN** 經瀏覽器登入完成後，於已連接的專案執行 speclink auth status
- **THEN** stdout 第二行為 `  Credential from: system keychain (login session, shared with the desktop app)`，不含 `device login`，exit code 為 0

#### Scenario: 多個 repo 的專案且已設定 repo

- **WHEN** 專案註冊 erp 與 crm 兩個 repo，remote section 的 repo 為 crm，以有效憑證執行 speclink auth status
- **THEN** stdout 含身分、憑證來源與 `Repo 'crm' is registered in this project`，exit code 為 0

#### Scenario: 多個 repo 的專案且沒有設定 repo

- **WHEN** 專案註冊 erp 與 crm 兩個 repo，remote section 沒有 repo，以有效憑證執行 speclink auth status
- **THEN** stdout 含身分、憑證來源，以及一行以 `! ` 開頭、含 `--repo`、erp 與 crm 的提示，exit code 為 0

#### Scenario: --json 的 payload 不變

- **WHEN** 金鑰圈存有登入憑證，於註冊多個 repo 的專案執行 speclink auth status --json
- **THEN** stdout 為單一 JSON 物件，含 user.name、user.handle 與值為 keychain_refresh 的 credentialSource，沒有其他欄位，也沒有 repo 的提示行

#### Scenario: 不是專案成員

- **WHEN** 憑證有效但使用者不是該專案的成員，執行 speclink auth status
- **THEN** exit code 非 0，stderr 為既有的 access denied 訊息，stdout 沒有身分與憑證來源

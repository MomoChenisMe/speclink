## ADDED Requirements

### Requirement: 用戶端以作業系統的信任庫驗證 Server 的憑證

CLI 與 Speclink Desktop 以 HTTPS 連到 Server 或登入服務時，SHALL 由作業系統的信任庫決定是否信任對方的憑證：簽發憑證的 CA 在作業系統的信任庫裡時連線 SHALL 成功，不論該 CA 是公開的 CA 還是組織自己的 CA；使用者 SHALL NOT 需要為此設定任何旗標、設定檔欄位或環境變數。這條規則 SHALL 一體適用用戶端發出的每一種連線：登入方式的查詢、身分查詢、瀏覽器登入的說明文件查詢與換 token、裝置登入的請求、握手與一般 API 請求、事件串流、狀態查詢（sync-state）。CLI 與 Speclink Desktop 對同一台 Server 的信任結果 SHALL 相同。

作業系統沒有提供任何 CA 時（例如沒有安裝 CA 憑證套件的精簡 Linux 容器），用戶端 SHALL 退回程式內建的公開 CA 清單，SHALL NOT 因此讓全部 HTTPS 連線失敗。以 http:// 開頭的連線 SHALL 不受本需求影響。

Server 的憑證被拒絕時（簽發的 CA 不在信任庫、憑證過期、主機名稱不符等），用戶端 SHALL 把它當成連線失敗處理：不讀快取、不重試、不退回本機模式。CLI SHALL 以非零 exit code 結束，並在 stderr 輸出一行英文訊息；這行訊息 SHALL 以 `server unreachable` 開頭，SHALL 含 `certificate` 一詞與憑證被拒絕的原因，SHALL 指出處理方法是把簽發憑證的 CA 加進作業系統的信任庫。原因是作業系統的驗證器回報的原因名稱；同一種情況在不同作業系統、或 Server 送出的憑證鏈不同時，名稱 MAY 不同，驗證器沒有給出具體名稱時是 `Other`。訊息 SHALL NOT 含原因名稱以外、取自對方憑證的文字。訊息的語言 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變，`--no-color` 下文字相同。Speclink Desktop SHALL 把這種失敗呈現為連線中斷，與其他連線失敗相同。登入服務的憑證被拒絕時，登入指令 SHALL 以非零 exit code 結束，訊息 SHALL 同樣含 `certificate` 一詞、被拒絕的原因與處理方法。

憑證以外的連線失敗（拒絕連線、逾時、DNS 解析失敗）的訊息 SHALL 維持既有的位元級輸出：`server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)`。憑證被拒絕的訊息是刻意新增的輸出：變更前它和其他連線失敗共用上面那一句，變更後改為指出憑證的那一句。動詞的 stdout 與 `--json` 輸出 SHALL 不因本需求改變。

#### Scenario: 組織的 CA 在信任庫裡時連線成功

- **WHEN** Server 的憑證由組織自己的 CA 簽發，該 CA 已裝進這台電腦的作業系統信任庫，使用者在連到這台 Server 的 checkout 執行 `speclink list`
- **THEN** 指令成功，exit code 為 0，輸出與連到公開 CA 簽發憑證的 Server 時相同

#### Scenario: 簽發的 CA 不在信任庫時指出憑證

- **WHEN** Server 的憑證由一個不在作業系統信任庫裡的 CA 簽發，使用者執行 `speclink list`
- **THEN** 指令以非零 exit code 結束，stdout 沒有變更清單，stderr 的訊息以 `server unreachable` 開頭、含 `certificate` 與被拒絕的原因，並指出把簽發的 CA 加進作業系統的信任庫

##### Example: 訊息的開頭與內容

| 失敗的原因 | stderr 訊息的開頭 | 訊息另外含有 |
| --- | --- | --- |
| 簽發的 CA 不在信任庫，Server 只送出自己的憑證（macOS、Linux） | `Error: server unreachable — ` | `certificate`、`UnknownIssuer`、`trust store` |
| 簽發的 CA 不在信任庫，Server 連根憑證一起送出（macOS） | `Error: server unreachable — ` | `certificate`、`Other`、`trust store` |
| Server 沒有在聽（拒絕連線） | `Error: server unreachable — check the connection url` | 與變更前逐字相同，不含 `certificate` |

#### Scenario: 其他連線失敗的訊息不變

- **WHEN** 連線的網址指向一個沒有程序在聽的埠，使用者執行 `speclink list`
- **THEN** stderr 的訊息與本需求引入之前逐位元相同，不含 `certificate` 一詞

#### Scenario: 每一種連線都用同一條信任規則

- **WHEN** Server 與它宣告的登入服務都使用組織的 CA 簽發的憑證，該 CA 在作業系統的信任庫裡，使用者依序執行瀏覽器登入、一個讀取動詞，並在 Speclink Desktop 開啟同一個遠端專案
- **THEN** 登入方式的查詢、說明文件查詢、換 token、握手、讀取請求、事件串流與狀態查詢全部成功，沒有任何一種連線因憑證失敗

#### Scenario: 作業系統沒有任何 CA 時退回內建清單

- **WHEN** 用戶端在一個作業系統沒有提供任何 CA 的環境執行，連到一台憑證由公開 CA 簽發的 Server
- **THEN** 連線成功，行為與本需求引入之前相同

#### Scenario: Speclink Desktop 把憑證被拒絕呈現為連線中斷

- **WHEN** Speclink Desktop 開啟一個遠端專案，而 Server 的憑證不被作業系統信任
- **THEN** 畫面顯示連線中斷，寫入動作停用，與 Server 沒有回應時的呈現相同

#### Scenario: http 連線不受影響

- **WHEN** 連線的網址以 http:// 開頭，指向本機的示範用 server
- **THEN** 連線的行為與本需求引入之前相同，不做任何憑證檢查

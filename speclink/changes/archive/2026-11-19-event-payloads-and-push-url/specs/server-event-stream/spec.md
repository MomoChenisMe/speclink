## ADDED Requirements

### Requirement: 宣告的串流網址的寫法與心跳的契約

握手宣告的 sse transport 的 url SHALL 是下列兩種寫法之一：沒有協定與主機、以 `/` 開頭的路徑（由用戶端接在該專案的 API 位址之後），或與 Server 同一個 origin 的完整網址。示範用 Server 宣告的 url SHALL 維持 `/events`。宿主經宿主套件宣告推送時，宣告的內容 SHALL 原樣出現在握手回應；宿主套件 SHALL NOT 改寫 url。

提供事件串流的一方（示範用 Server，或自行提供串流的宿主）SHALL 在串流閒置時送出 SSE 的註解行當作心跳。示範用 Server 的心跳間隔預設 SHALL 是 15 秒。用戶端在 45 秒內沒有收到任何位元組（心跳也算）時 SHALL 把連線視為中斷並重新連線；因此心跳間隔 SHALL 小於 45 秒，串流才不會被反覆重連。

示範用 Server 的設定檔的 `events.heartbeat_secs` SHALL 小於 45。值是 45 以上時，Server SHALL 在綁定任何埠之前以設定錯誤結束（exit code 非零），stderr 的訊息 SHALL 指出設定檔的路徑、欄位 `events.heartbeat_secs` 與規則 `must be less than 45`。值是 44 以下、或沒有設定這個欄位（預設 15）時，行為 SHALL 與引入本需求之前相同。

使用者文件 SHALL 寫明這三件事：url 的兩種寫法與同 origin 的限制、15 秒與 45 秒這兩個數字、以及事件只指出哪個資源變了而用戶端收到後重新讀取。

#### Scenario: 示範用 Server 的宣告不變

- **WHEN** 對示範用 Server 完成握手
- **THEN** 回應宣告的 sse transport 的 url 是 `/events`、resume 是 true，polling 宣告的 url 是 `/sync-state`

#### Scenario: 宿主宣告的 url 原樣出現在握手

- **WHEN** 宿主建立宿主套件時宣告 sse transport 的 url 是 `https://specs.example.com/push/acme`，用戶端完成握手
- **THEN** 握手回應的 transports 含 type 為 sse、url 為 `https://specs.example.com/push/acme` 的一項

#### Scenario: 閒置的串流在 45 秒內有心跳

- **WHEN** 訂閱者連上示範用 Server（預設設定）的事件串流，之後沒有任何寫入
- **THEN** 訂閱者每隔約 15 秒收到一行註解；連線沒有因為閒置被用戶端判定為中斷

#### Scenario: 示範用 Server 拒絕 45 秒以上的心跳設定

- **WHEN** 設定檔寫 `events.heartbeat_secs: 45`，啟動示範用 Server
- **THEN** Server 沒有綁定任何埠就結束，exit code 非零；stderr 含設定檔路徑與 `'events.heartbeat_secs' must be less than 45`

##### Example: 各種 heartbeat_secs 的結果

| 設定 | 結果 |
| --- | --- |
| 沒有 `events` 區段 | 啟動，心跳每 15 秒 |
| `heartbeat_secs: 5` | 啟動，心跳每 5 秒 |
| `heartbeat_secs: 44` | 啟動，心跳每 44 秒 |
| `heartbeat_secs: 45` | 不啟動，訊息指出 `events.heartbeat_secs` 與 `must be less than 45` |
| `heartbeat_secs: 60` | 不啟動，同一則訊息 |

#### Scenario: 超過 45 秒沒有資料時用戶端重連

- **WHEN** 一條串流連上之後超過 45 秒沒有送出任何位元組
- **THEN** 用戶端把這條連線視為中斷，以輪詢比對是否有錯過的變更，並重新訂閱

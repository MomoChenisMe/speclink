## ADDED Requirements

### Requirement: 事件串流的網址取自握手的宣告

Speclink Desktop 建立 SSE 訂閱時 SHALL 使用握手回應宣告的 sse transport 的 url，SHALL NOT 使用寫死的路徑。url 的解析規則：

- url 沒有協定與主機（例如 `/events`）時，SHALL 把它接在這個專案的 API 位址之後（`{origin}/api/speclink/v1/projects/{project}` 加上 url），與輪詢宣告的 url 用同一個規則。
- url 是完整的網址（以 `http://` 或 `https://` 開頭）時，SHALL 只在它的 origin（協定、主機、埠）與這條連線的 Server 相同時使用。

Desktop 實際連線的網址 SHALL 是通過 origin 檢查的那一次解析的結果，以正規寫法表示：主機轉成小寫、與協定預設值相同的埠省略、路徑裡的 `.` 與 `..` 化簡。Desktop SHALL NOT 把宣告的原始字串另外再解析一次來連線——檢查時看到的主機與實際連線的主機 SHALL 是同一個。宣告的寫法已經是正規寫法時，連線的網址與宣告的逐字相同。

握手宣告多個 sse transport 時 SHALL 使用第一個。下列情況 Desktop SHALL NOT 建立 SSE 訂閱，SHALL 改以輪詢（/sync-state 的 ETag）收斂，且 SHALL NOT 把這當成錯誤呈現給使用者：握手沒有宣告 sse transport（含只宣告 websocket）；宣告的完整網址與 Server 不同 origin；url 無法解析。Desktop SHALL NOT 把這條連線的存取憑證送到與 Server 不同 origin 的網址。

訂閱建立之後的行為——失效提示觸發重讀、多個 session 共用單一訂閱、中斷時以輪詢收斂並帶 Last-Event-ID 續傳、reset 信號的處理、認證失效的處理——SHALL 與既有需求相同，只有連線的網址改由宣告決定。

#### Scenario: 相對的 url 接在專案的 API 位址之後

- **WHEN** Server 位於 `https://specs.example.com`，專案是 acme，握手宣告 sse transport 的 url 是 `/events`
- **THEN** Desktop 訂閱 `https://specs.example.com/api/speclink/v1/projects/acme/events`

##### Example: 宣告的 url 與實際訂閱的網址

Server 位於 `https://specs.example.com`，專案是 acme。

| 宣告的 url | Desktop 的行為 |
| --- | --- |
| `/events` | 訂閱 `https://specs.example.com/api/speclink/v1/projects/acme/events` |
| `/stream/changes` | 訂閱 `https://specs.example.com/api/speclink/v1/projects/acme/stream/changes` |
| `https://specs.example.com/push/acme` | 訂閱 `https://specs.example.com/push/acme` |
| `https://SPECS.Example.com:443/push/acme` | 訂閱 `https://specs.example.com/push/acme`（同 origin，連線用正規寫法） |
| `https://push.example.com/acme` | 不訂閱，改用輪詢（不同主機） |
| `http://specs.example.com/push/acme` | 不訂閱，改用輪詢（協定不同） |
| `https://specs.example.com:8443/push` | 不訂閱，改用輪詢（埠不同） |
| （只宣告 websocket） | 不訂閱，改用輪詢 |

#### Scenario: 同 origin 的完整網址可以使用

- **WHEN** 宿主在握手宣告 sse transport 的 url 是 `https://specs.example.com/push/acme`，而 Server 位於 `https://specs.example.com`
- **THEN** Desktop 訂閱該網址並帶上存取憑證；另一個用戶端寫入後，Desktop 的畫面不經手動重新整理就更新

#### Scenario: 寫法不同但同 origin 的網址以正規寫法連線

- **WHEN** 宿主在握手宣告 sse transport 的 url 是 `https://SPECS.Example.com:443/push/acme`，而 Server 位於 `https://specs.example.com`
- **THEN** Desktop 訂閱 `https://specs.example.com/push/acme`，並帶上存取憑證

#### Scenario: 不同 origin 時不送出憑證並改用輪詢

- **WHEN** 握手宣告 sse transport 的 url 是 `https://push.example.com/acme`，而 Server 位於 `https://specs.example.com`
- **THEN** Desktop 沒有對 `push.example.com` 發出任何請求；畫面靠輪詢更新；使用者沒有看到錯誤訊息

#### Scenario: 連到示範用 server 的行為不變

- **WHEN** Desktop 連到示範用 server（握手宣告的 url 是 `/events`）
- **THEN** 訂閱的網址與引入本需求之前相同，即時更新照常運作

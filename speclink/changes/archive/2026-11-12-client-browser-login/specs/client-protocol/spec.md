## ADDED Requirements

### Requirement: 登入方式宣告的 payload

Client Protocol SHALL 定義登入方式的探索：對 API 根底下的 auth-methods 路徑發出 GET，請求 SHALL NOT 需要任何憑證或版本標頭。回應的 payload 為 camelCase：`methods`（string 陣列，值為 `browser`、`device`、`token`，代表 server 支援的登入方式）與選填的 `browser`（物件）。`browser` 物件含 `issuer`（string，登入服務的發行者網址）、`clientId`（string，用戶端在登入服務登記的識別）與選填的 `scopes`（string 陣列，缺席讀作空陣列）。這個型別 SHALL 在 protocol 的單一定義處宣告，用戶端與 server 端 SHALL 共用它。用戶端 SHALL 略過 `methods` 裡不認得的值，SHALL 把缺席的 `browser` 物件讀作沒有瀏覽器登入的設定。沒有這個路徑的 server（回應 404 或 405）SHALL 被用戶端視為「沒有宣告」，這個情況 SHALL NOT 被視為錯誤。握手回應既有的欄位 SHALL 不變。

#### Scenario: 完整的宣告

- **WHEN** 用戶端解析 `{"methods":["browser","device"],"browser":{"issuer":"https://id.example.com","clientId":"speclink","scopes":["openid"]}}`
- **THEN** 得到兩種方式，瀏覽器登入的發行者網址是 https://id.example.com、用戶端識別是 speclink、scope 是 openid

#### Scenario: scopes 缺席

- **WHEN** 用戶端解析的 browser 物件只有 issuer 與 clientId
- **THEN** scope 讀作空陣列，沒有解析錯誤

#### Scenario: 不認得的方式

- **WHEN** 用戶端解析 `{"methods":["passkey","token"]}`
- **THEN** 得到一種方式 token，沒有解析錯誤

#### Scenario: 序列化為 camelCase

- **WHEN** server 端序列化一份含瀏覽器登入設定的宣告
- **THEN** JSON 的鍵是 methods、browser、issuer、clientId、scopes

#### Scenario: 舊 server 沒有這個路徑

- **WHEN** 用戶端對沒有 auth-methods 路徑的 server 發出探索，server 回應 404
- **THEN** 用戶端得到「沒有宣告」的結果而不是錯誤

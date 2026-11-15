## MODIFIED Requirements

### Requirement: 保留政策與 reset 訊號
<!-- BEFORE: 只有 Last-Event-ID 落在已清理範圍時送 reset；沒有規定 Last-Event-ID 大於目前最大序號的情況 -->

server SHALL 依保留政策 ack outbox（每 scope 保留最近 N 筆，N 可組態、有預設值），讓 driver 得以清理更舊事件。重連的 Last-Event-ID 落在已清理範圍時，server SHALL 立即送出明確的 reset 訊號事件（獨立的 SSE 事件種類）再繼續推播新事件；SHALL NOT 猜測或重建已清理的事件。重連的 Last-Event-ID 大於該 scope 目前最大的序號時（例如還原之後事件序號重新開始），server SHALL 同樣立即送出 reset 訊號，再從目前最大的序號之後繼續推播新事件，SHALL NOT 等到序號超過該 Last-Event-ID 才開始推播。client 據 reset 以 Query 與 ETag 全部重新收斂的路徑 SHALL 可用（既有輪詢端點不受本能力影響）。

#### Scenario: 過期 cursor 得到 reset

- **WHEN** 某 scope 已 ack 清理至序號 m，client 以小於 m 的 Last-Event-ID 重連
- **THEN** client 先收到 reset 訊號事件，隨後的新寫入照常推播；server 未嘗試重送已清理序號

#### Scenario: 漏光事件仍能收斂

- **WHEN** client 收到 reset 後改以既有輪詢端點與查詢路由重讀
- **THEN** 重讀結果反映全部已 commit 的變更；後續可重新訂閱自最新序號續聽

#### Scenario: Last-Event-ID 比 server 的序號新

- **WHEN** 某 scope 目前最大的事件序號是 3，client 以 Last-Event-ID 為 500 重連，之後發生一筆寫入
- **THEN** client 先收到 reset 訊號，再收到序號 4 的事件

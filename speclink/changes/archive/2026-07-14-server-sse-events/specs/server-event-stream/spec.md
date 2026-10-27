## ADDED Requirements

### Requirement: SSE 串流自 outbox 且事件是 invalidation hint

server SHALL 提供 project-scoped 的 SSE 事件端點，沿用既有 bearer 前置與 binding 裁決（未認證 401、非成員 403，與其他路由一致）。推播內容 SHALL 出自該 scope 的 outbox 讀取：SSE 事件識別 SHALL 為 outbox 序號，data SHALL 為 protocol 定義的 invalidation DTO——僅攜事件識別、scope 種類、資源識別與 revision，SHALL NOT 承載規格文件內容。寫入 commit 成功後 SHALL 向該 scope 全部訂閱者推播對應事件；對映不了資源類別的領域事件 SHALL 以 unknown 類別照發，SHALL NOT 靜默略過。連線期間 SHALL 以固定間隔送出 SSE 註解心跳。

#### Scenario: 寫入即推播 invalidation

- **WHEN** client A 訂閱 /events 期間 client B 成功執行 task done
- **THEN** client A 收到一筆事件：識別為該筆 outbox 序號、資源識別為該 change、revision 為 commit 後值；data 不含任務文件內容

#### Scenario: 多訂閱者同事件

- **WHEN** 兩個 client 同時訂閱同一 scope 期間發生一筆寫入
- **THEN** 兩者各收到恰一筆相同識別的事件

#### Scenario: 未授權連線拒於門外

- **WHEN** 以無效 token 或非成員身分連線 /events
- **THEN** 回應與其他路由一致的 401/403 三元組；不建立串流

---
### Requirement: resume 續傳不漏不重

帶 Last-Event-ID 的重連 SHALL 自該序號之後續傳：重連前後收到的事件序列 SHALL 與不斷線時一致——無缺漏、無重複、序號單調遞增。Last-Event-ID 格式非法 SHALL 視同未帶（自最新事件之後開始）。續傳與即時推播 SHALL 讀自同一 outbox 序列，SHALL NOT 出現兩者順序分叉。

#### Scenario: 斷線期間事件補齊

- **WHEN** client 收到序號 n 的事件後斷線，期間發生三筆寫入，client 以 Last-Event-ID 為 n 重連
- **THEN** client 依序收到 n 之後的三筆事件各恰一次，序號單調遞增

---
### Requirement: 保留政策與 reset 訊號

server SHALL 依保留政策 ack outbox（每 scope 保留最近 N 筆，N 可組態、有預設值），讓 driver 得以清理更舊事件。重連的 Last-Event-ID 落在已清理範圍時，server SHALL 立即送出明確的 reset 訊號事件（獨立的 SSE 事件種類）再繼續推播新事件；SHALL NOT 猜測或重建已清理的事件。client 據 reset 以 Query 與 ETag 全量收斂的路徑 SHALL 可用（既有輪詢端點不受本能力影響）。

#### Scenario: 過期 cursor 得到 reset

- **WHEN** 某 scope 已 ack 清理至序號 m，client 以小於 m 的 Last-Event-ID 重連
- **THEN** client 先收到 reset 訊號事件，隨後的新寫入照常推播；server 未嘗試重送已清理序號

#### Scenario: 漏光事件仍能收斂

- **WHEN** client 收到 reset 後改以既有輪詢端點與查詢路由重讀
- **THEN** 重讀結果反映全部已 commit 的變更；後續可重新訂閱自最新序號續聽

---
### Requirement: 慢消費者有界處置

每條 SSE 連線的送出緩衝 SHALL 有界（可組態、有預設值）；緩衝溢出 SHALL 使 server 斷開該連線，SHALL NOT 為單一訂閱者無界堆積記憶體，且 SHALL NOT 影響其他訂閱者收事件。被斷開的 client 以 Last-Event-ID 重連 SHALL 依 resume 與 reset 規則恢復。

#### Scenario: 慢消費者不拖累他人

- **WHEN** 兩個訂閱者之一停止讀取直到其緩衝溢出，期間持續發生寫入
- **THEN** 停讀者被斷線；另一訂閱者持續即時收到全部事件無缺漏

---
### Requirement: binding 宣告 sse transport

/binding 的 capabilities SHALL 宣告 events transports 含 sse（含事件端點 url 與 resume 支援為真），polling 宣告 SHALL 維持不變。宣告 SHALL 與實際服務一致：宣告的 url 即本能力的事件端點。invalidation DTO SHALL 定義於 protocol crate（camelCase、可匯出 JSON Schema、序列化往返穩定）。

#### Scenario: 宣告與服務一致

- **WHEN** 完成 handshake 取得 capabilities 後對宣告的 sse url 建立訂閱
- **THEN** 宣告含 type 為 sse、resume 為 true 的 transport 與不變的 polling 宣告；該 url 的訂閱可收到 invalidation 事件
